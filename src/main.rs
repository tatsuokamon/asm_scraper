use std::{env, sync::Arc, time::Duration};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncConnectionConfig;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use crate::{
    engine::ready_router,
    redis_window::{MultiplexedAcquireConfig, OnetimeConfig, PoolAcquireConfig, StreamConfig, redis_request_producer},
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
    };
}

macro_rules! get_env_with_parsing {
    ($oprand:ident, $keyword:expr, $dest:ty) => {
        let $oprand = env::var($keyword)
            .expect(&format!("failed to load: {}", $keyword))
            .parse::<$dest>()
            .expect(&format!("failed to parse: {}", $keyword));
    };
}

fn backoff_algo_temporary(current: Duration) -> Duration {
    current * 2
}

#[tokio::main]
async fn main() {
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
        "REDIS_POOL_CONNECTION_TIMEOUT",
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
    get_env_with_parsing!(channel_buf, "CHANNEL_BUF", usize);
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
        result_keyword
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
    );

    get_env!(addr, "EXPOSED_URL");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("falied to bind TcpListener");

    axum::serve(listener, router).await.expect("failed to serve");
}
