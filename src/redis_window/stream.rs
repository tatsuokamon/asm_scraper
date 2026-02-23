use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::{
    sync::{Arc, atomic},
    time::Duration,
};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{
    redis_communication::RedisRequest,
    redis_window::{
        acquire::{MultiplexedAcquireConfig, PoolAcquireConfig, acquire_conn},
        err::{RedisHandleErr, RedisWindowErr},
        redis_job::RedisJob,
    },
};

#[derive(Clone)]
pub struct StreamConfig {
    pub channel_buf: usize, // url_tx_channel,
    pub blocking_time: f64,
    pub how_long_to_wait: u64, // redis_request_timeout
    pub hash_get_retry: usize,
    pub result_keyword: String
}

pub struct StreamState {
    inflight: atomic::AtomicUsize,
    finished: atomic::AtomicBool,
}

impl StreamState {
    fn new() -> Self {
        Self {
            inflight: atomic::AtomicUsize::new(0),
            finished: atomic::AtomicBool::new(false),
        }
    }

    fn inflight_sub(&self, value: usize) {
        self.inflight.fetch_sub(value, atomic::Ordering::Release);
    }

    fn inflight_add(&self, value: usize) {
        self.inflight.fetch_add(value, atomic::Ordering::Release);
    }

    fn store_finished_state(&self, finished: bool) {
        self.finished.store(finished, atomic::Ordering::Release);
    }

    fn finished(&self) -> bool {
        if !self.finished.load(atomic::Ordering::Acquire) {
            return false;
        };

        self.inflight.load(atomic::Ordering::Acquire) == 0
    }
}

async fn create_stream_result_part(
    result_tx: Sender<Result<String, RedisHandleErr>>,
    result_keyword: &String,
    mut id_rx: Receiver<String>,

    pool: Arc<Pool<RedisConnectionManager>>,
    pool_acquire_config: Arc<PoolAcquireConfig>,
    hget_retry: usize,
) -> () {
    let mut conn;
    match acquire_conn(pool_acquire_config.clone(), &pool, None).await {
        Some(c) => {
            conn = c;
        }
        None => {
            if let Err(e) = result_tx
                .send(Err(RedisHandleErr::RedisNoneConnection))
                .await
            {
                tracing::error!("{e}");
            };
            return;
        }
    };

    while let Some(received_id) = id_rx.recv().await {
        let mut tempt = 0;
        while tempt < hget_retry {
            match conn
                .hget::<&String, &String, String>(result_keyword, &received_id)
                .await
            {
                Ok(hash_got) => {
                    if let Err(e) = result_tx.send(Ok(hash_got)).await {
                        tracing::error!("{e}");
                    }
                }
                Err(e) => {
                    tracing::error!("{e}");

                    match acquire_conn(pool_acquire_config.clone(), &pool, None).await {
                        Some(c) => {
                            conn = c;
                        }
                        None => {
                            if let Err(e) = result_tx
                                .send(Err(RedisHandleErr::RedisNoneConnection))
                                .await
                            {
                                tracing::error!("{e}");
                            };
                            return;
                        }
                    };
                    tempt += 1;
                }
            }
        }
    }
}

async fn create_stream_id_getter_part(
    result_tx: Sender<Result<String, RedisHandleErr>>,
    redis_job_id: String,
    id_tx: Sender<String>,

    client: Arc<redis::Client>,
    client_acquire_config: Arc<MultiplexedAcquireConfig>,

    stream_state: Arc<StreamState>,

    blocking_time: f64,
    how_long_to_wait: u64,
) -> () {
    let result_tx_for_sleep = result_tx.clone();
    let result_tx_for_main = result_tx;

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(how_long_to_wait)) => {
            if let Err(e) = result_tx_for_sleep.send(Err(RedisHandleErr::Timeout)).await {
                tracing::error!("{e}");
            };
        },
        _ = async move {
            let mut conn;

            match acquire_conn(client_acquire_config.clone(), &client, None).await {
                Some(got_conn) => {
                    conn = got_conn;
                },
                None => {
                    if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::RedisNoneConnection)).await {
                        tracing::error!("{e}");
                    };
                    return;
                }
            };

            loop {
                if stream_state.finished() {
                    break;
                }

                match conn.brpop::<&String, (String, String)>(&redis_job_id, blocking_time).await {
                    Ok(received) => {
                        stream_state.inflight_sub(1); // idが戻ってきた時点でstate側の終了条件とする (完全にresultを返す時点ではないことに注意)
                        let (_, task_id) = received;
                        if let Err(e) = id_tx.send(task_id).await {
                            tracing::error!("{e}");
                            if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::IDSenderErr)).await {
                                tracing::error!("{e}");
                                continue;
                            }
                        }

                    },
                    Err(e) => {
                        tracing::error!("{e}");
                        match acquire_conn(client_acquire_config.clone(), &client, None).await {
                            Some(c) => {
                                conn = c;
                            },
                            None => {
                                if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::RedisNoneConnection)).await {
                                    tracing::error!("{e}");
                                }
                            }
                        }
                    }
                }
            } // loop out
        } => {}
    }
}

async fn create_stream_request_part<RR>(
    mut url_rx: Receiver<String>,
    req_tx: Sender<String>,
    result_tx: Sender<Result<String, RedisHandleErr>>,
    mut redis_job: RedisJob,
    stream_state: Arc<StreamState>,
) -> ()
where
    RR: RedisRequest + serde::ser::Serialize,
{
    while let Some(url) = url_rx.recv().await {
        let req_string = {
            let redis_req: RR = redis_job.generate_redis_request(url.clone());
            serde_json::to_string(&redis_req).unwrap()
        };

        if let Err(e) = req_tx.send(req_string).await {
            tracing::error!("{e}");
            if let Err(e) = result_tx.send(Err(RedisHandleErr::RequestSenderErr)).await {
                tracing::error!("{e}");
            };
            continue;
        }

        stream_state.inflight_add(1);
    } // loop out
    //

    stream_state.store_finished_state(true);
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
    req_tx: Sender<String>, // redis_request sender
    stream_config: Arc<StreamConfig>,
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
                req_tx,
                result_tx_moved_to_req_thread,
                redis_job,
                stream_state_moved_to_req_thread
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
