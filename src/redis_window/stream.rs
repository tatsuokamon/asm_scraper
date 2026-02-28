use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::{
    sync::{Arc, atomic},
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{
    redis_communication::RedisRequest,
    redis_window::{
        acquire::{MultiplexedAcquireConfig, PoolAcquireConfig},
        err::{RedisHandleErr, RedisWindowErr},
        redis_job::RedisJob,
        stream_req_part::{RequestContract, create_stream_request_part},
        stream_result_part::create_stream_result_part,
        stream_id_getter_part::create_stream_id_getter_part
    },
};

#[derive(Clone)]
pub struct StreamConfig {
    pub channel_buf: usize, // url_tx_channel,
    pub blocking_time: f64,
    pub how_long_to_wait: u64, // redis_request_timeout
    pub hash_get_retry: usize,
    pub result_keyword: String,
}

pub struct StreamState {
    inflight: atomic::AtomicUsize,
    finished: atomic::AtomicBool,
    abort: atomic::AtomicBool,
}

impl StreamState {
    fn new() -> Self {
        Self {
            inflight: atomic::AtomicUsize::new(0),
            finished: atomic::AtomicBool::new(false),
            abort: atomic::AtomicBool::new(false),
        }
    }

    pub fn inflight_sub(&self, value: usize) {
        self.inflight.fetch_sub(value, atomic::Ordering::Release);
    }

    pub fn inflight_add(&self, value: usize) {
        self.inflight.fetch_add(value, atomic::Ordering::Release);
    }

    pub fn store_finished_state(&self, finished: bool) {
        self.finished.store(finished, atomic::Ordering::Release);
    }
    pub fn abort(&self) {
        self.abort.store(true, atomic::Ordering::Release);
    }

    pub fn finished(&self) -> bool {
        if self.abort.load(atomic::Ordering::Acquire) {
            return true;
        }

        if !self.finished.load(atomic::Ordering::Acquire) {
            return false;
        };

        self.inflight.load(atomic::Ordering::Acquire) == 0
    }
}

pub async fn create_stream<RR>(
    // control thread;
    set: &mut JoinSet<()>,
    token: CancellationToken,

    pool: Arc<Pool<RedisConnectionManager>>,
    client: Arc<redis::Client>,
    // client connection config
    multiplexed_acquire_config: Arc<MultiplexedAcquireConfig>,
    pool_acquire_config: Arc<PoolAcquireConfig>,
    stream_config: Arc<StreamConfig>,

    req_contract: RequestContract
) -> Result<(Sender<String>, Receiver<Result<String, RedisHandleErr>>), RedisWindowErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    pool.get().await?; // check pool health; if something is wrong with it return err;
    client
        .get_multiplexed_async_connection_with_config(&multiplexed_acquire_config.connection_config)
        .await?;

    let (url_tx, url_rx) = tokio::sync::mpsc::channel(stream_config.channel_buf);
    let (result_tx, result_rx) = tokio::sync::mpsc::channel(stream_config.channel_buf);
    let redis_job = RedisJob::new();
    let job_id = redis_job.get_id();

    let stream_state = Arc::new(StreamState::new());

    let stream_state_moved_to_req_thread = stream_state.clone();
    let result_tx_moved_to_req_thread = result_tx.clone();
    let token_moved_to_req_thread = token.child_token();

    set.spawn(async move {
        tokio::select! {
            _ = token_moved_to_req_thread.cancelled() => {},
            _ = create_stream_request_part::<RR>(
                url_rx,
                result_tx_moved_to_req_thread,
                redis_job,
                stream_state_moved_to_req_thread,
                req_contract
            ) => {}
        }
    });

    let stream_state_moved_to_id_getter_thread = stream_state;
    let result_tx_moved_to_id_getter_thread = result_tx.clone();
    let (id_tx, id_rx) = tokio::sync::mpsc::channel(stream_config.channel_buf);

    let blocking_time = stream_config.blocking_time;
    let how_long_to_wait = stream_config.how_long_to_wait;
    let hash_get_retry = stream_config.hash_get_retry;

    set.spawn(async move {
        _ = create_stream_id_getter_part(
            result_tx_moved_to_id_getter_thread,
            job_id,
            id_tx,
            client,
            multiplexed_acquire_config,
            stream_state_moved_to_id_getter_thread,
            blocking_time,
            how_long_to_wait,
        )
        .await;
    });

    let result_tx_moved_to_result_thread = result_tx;
    let id_rx_moved_to_result_thread = id_rx;
    set.spawn(async move {
        _ = create_stream_result_part(
            result_tx_moved_to_result_thread,
            &stream_config.result_keyword,
            id_rx_moved_to_result_thread,
            pool,
            pool_acquire_config,
            hash_get_retry,
        )
        .await
    });

    Ok((url_tx, result_rx))
}
