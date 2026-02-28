use std::{env, sync::Arc, time::Duration};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncConnectionConfig;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::{
    engine::{EngineConfig, ready_router},
    redis_window::{
        MultiplexedAcquireConfig, OnetimeConfig, PoolAcquireConfig, StreamConfig,
        redis_request_producer,
    },
};

mod db_executor;
mod engine;
mod entity;
mod model;
mod redis_communication;
mod redis_window;

macro_rules! get_env {
    ($operand:ident, $keyword:expr) => {
        let $operand = env::var($keyword).expect(&format!("failed to load :{}", $keyword));
        tracing::info!("config: {}: {}", $keyword, &$operand);
    };
}

macro_rules! get_env_with_parsing {
    ($operand:ident, $keyword:expr, $dest:ty) => {
        let $operand = env::var($keyword)
            .expect(&format!("failed to load: {}", $keyword))
            .parse::<$dest>()
            .expect(&format!("failed to parse: {}", $keyword));
        tracing::info!("config: {}: {}", $keyword, &$operand);
    };
}

fn backoff_algo_temporary(current: Duration) -> Duration {
    current.mul_f64(1.2_f64)
}

// need to be defined >>>
// 1: REDIS_URL: communication target redis url: example(redis://lcalhost:7)
// 2: BACKOFF_INIT: how long to wait after connection to redis server went bad: secs
// 3: REDIS_RETRY: how many times to try to reconnect after connection to redis server went bad : secs
// 4: META_REQUEST_Q_KEYWORD: what keyword do you use when send request of meta to redis server: ()
// 5: DETAIL_REQUEST_Q_KEYWORD: what keyword do you use when send request of detail to redis server: ()
// 6: TAG_REQUEST_Q_KEYWORD: what keyword do you use when send request of tag to redis server: ()
// 7: IDX_REQUEST_Q_KEYWORD: what keyword do you use when send request of idx to redis server: ()
// 8: RESULT_KEYWORD: what keyword do you use when get response from hash : ()
// 9: REDIS_POOL_MAX_SIZE: max pool connection: ()
// 10: REDIS_POOL_CONNECTION_TIMEOUT: pool connection timeout : ()
// 11: REDIS_REQUEST_CHANNEL_BUFFER : redis request channel buf : ()
//
// below env is about upper ray service
// 12: BLOCKING_TIME: how long to wait per one connection : ()
// 13: CHANNEL_BUFFER: the buffer of channel used when redis stream: ()
// 14: HOW_LONG_TO_WAIT: stream max time  : if exceeded, finish
// 15: HASH_GET_RETRY: how many times to try to get from redis : ()
//
// below env is about SSE
// 16: COUNT_CHANNEL_BUFFER: how many times to try to get from redis : ()
// 17: SSE_CHANNEL_BUFFER: how many times to try to get from redis : ()
//
//
// 18: DB_URL: db_url : assumed to be postgres://...
// 19: EXPOSED_HOST: exposed http host: (example: localhost:80)

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    dotenv::dotenv().expect("failed to load .env");

    // ready redis
    get_env!(redis_url, "REDIS_URL");
    let red_client = redis::Client::open(redis_url.clone()).expect("failed to open redis client");

    get_env_with_parsing!(backoff_init_count, "BACKOFF_INIT", u64);
    get_env_with_parsing!(redis_retry, "REDIS_RETRY", i32);

    let connection_config = AsyncConnectionConfig::new().set_response_timeout(None);
    let multiplexed_config = Arc::new(MultiplexedAcquireConfig {
        retry: redis_retry,
        connection_config,
        backoff_init: Duration::from_secs(backoff_init_count),
        backoff_algo: Arc::new(backoff_algo_temporary),
    });

    let pool_config = Arc::new(PoolAcquireConfig {
        retry: redis_retry,
        backoff_init: Duration::from_secs(backoff_init_count),
        backoff_algo: Arc::new(backoff_algo_temporary),
    });

    get_env!(meta_req_q_keyword, "META_REQUEST_Q_KEYWORD");
    get_env!(detail_req_q_keyword, "DETAIL_REQUEST_Q_KEYWORD");
    get_env!(tag_req_q_keyword, "TAG_REQUEST_Q_KEYWORD");
    get_env!(idx_req_q_keyword, "IDX_REQUEST_Q_KEYWORD");
    get_env!(result_keyword, "RESULT_KEYWORD");

    get_env_with_parsing!(redis_pool_max_size, "REDIS_POOL_MAX_SIZE", u32);
    get_env_with_parsing!(
        redis_pool_connection_timeout,
        "REDIS_POOL_CONNECTION_TIMEOUT",
        u64
    );
    get_env_with_parsing!(
        redis_request_channel_buf,
        "REDIS_REQUEST_CHANNEL_BUFFER",
        usize
    );
    let redis_manager =
        RedisConnectionManager::new(redis_url).expect("failed to create redis manager");
    let redis_pool = Arc::new(
        Pool::builder()
            .max_size(redis_pool_max_size)
            .connection_timeout(Duration::from_secs(redis_pool_connection_timeout))
            .build(redis_manager)
            .await
            .expect("failed to create redis pool"),
    );

    let mut base_set = JoinSet::new();
    let base_token = CancellationToken::new();

    macro_rules! invoke_tx {
        ($tx:ident, $keyword:expr) => {
            let $tx = redis_request_producer(
                $keyword,
                &mut base_set,
                base_token.child_token(),
                redis_pool.clone(),
                pool_config.clone(),
                redis_request_channel_buf,
            )
            .await
            .expect("failed to invoke redis request");
        };
    }

    invoke_tx!(meta_tx, meta_req_q_keyword);
    invoke_tx!(detail_tx, detail_req_q_keyword);
    invoke_tx!(_tag_tx, tag_req_q_keyword);
    invoke_tx!(idx_tx, idx_req_q_keyword);

    get_env!(db_url, "DB_URL");
    let db = sea_orm::Database::connect(db_url)
        .await
        .expect("failed to connect database");

    get_env_with_parsing!(blocking_time, "BLOCKING_TIME", f64);
    get_env_with_parsing!(channel_buf, "CHANNEL_BUFFER", usize);
    get_env_with_parsing!(how_long_to_wait, "HOW_LONG_TO_WAIT", u64);
    get_env_with_parsing!(hash_get_retry, "HASH_GET_RETRY", usize);

    let stream_conf = StreamConfig {
        channel_buf,
        blocking_time,
        how_long_to_wait,
        hash_get_retry,
        result_keyword: result_keyword.clone(),
    };

    let onetime_conf = OnetimeConfig {
        blocking_time,
        how_long_to_wait,
        hash_get_retry,
        result_keyword,
    };

    get_env_with_parsing!(count_channel_buf, "COUNT_CHANNEL_BUFFER", usize);
    get_env_with_parsing!(sse_channel_buf, "SSE_CHANNEL_BUFFER", usize);

    let engine_conf = EngineConfig {
        count_channel_buf,
        sse_channel_buf,
    };

    let router = ready_router(
        redis_pool,
        red_client,
        reqwest::Client::new(),
        db,
        idx_tx,
        detail_tx,
        meta_tx,
        multiplexed_config,
        pool_config,
        Arc::new(stream_conf),
        Arc::new(onetime_conf),
        engine_conf,
    );

    get_env!(addr, "EXPOSED_HOST");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("falied to bind TcpListener");

    axum::serve(listener, router)
        .await
        .expect("failed to serve");
}
