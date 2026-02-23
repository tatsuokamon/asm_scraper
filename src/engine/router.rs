use std::sync::Arc;

use crate::{
    engine::{
        process_meta::finding_meta_process,
        state::{EngineState, EngineStateStruct},
    },
    redis_communication::BasicRedisReq,
    redis_window::{MultiplexedAcquireConfig, OnetimeConfig, PoolAcquireConfig, StreamConfig},
};
use axum::{Router, routing::get};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use tokio::sync::mpsc::Sender;

pub fn ready_router(
    pool: Arc<Pool<RedisConnectionManager>>,
    red_client: redis::Client,
    http_client: reqwest::Client,
    db: sea_orm::DatabaseConnection,
    idx_tx: Sender<String>,
    url_tx: Sender<String>,
    meta_tx: Sender<String>,

    multiplexed_acquire_config: Arc<MultiplexedAcquireConfig>,
    pool_acquire_confing: Arc<PoolAcquireConfig>,
    stream_conf: Arc<StreamConfig>,
    onetime_conf: Arc<OnetimeConfig>,
) -> Router {
    let state: EngineState = Arc::new(EngineStateStruct {
        pool: pool,
        red_client: Arc::new(red_client),
        http_client: Arc::new(http_client),
        db,
        idx_tx,
        meta_tx,
        url_tx,
        multiplexed_acquire_config: multiplexed_acquire_config,
        pool_acquire_config: pool_acquire_confing,
        stream_config: stream_conf,
        onetime_config: onetime_conf,
    });

    Router::new()
        .route("/meta", get(finding_meta_process))
        .with_state(state)
}
