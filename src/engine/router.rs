use std::sync::Arc;

use crate::{
    engine::{
        EngineConfig,
        find_detail::{self, find_detail_handler},
        find_roughs::find_roughs_handler,
        process_meta::scraping_meta_process,
        state::{EngineState, EngineStateStruct},
        update_tag::update_handler,
    },
    redis_communication::{BasicRedisReq, RedisRequest},
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
    update_tx: Sender<String>,

    multiplexed_acquire_config: Arc<MultiplexedAcquireConfig>,
    pool_acquire_config: Arc<PoolAcquireConfig>,
    stream_conf: Arc<StreamConfig>,
    onetime_conf: Arc<OnetimeConfig>,
    engine_conf: EngineConfig,
) -> Router {
    let state: EngineState = Arc::new(EngineStateStruct {
        pool,
        red_client: Arc::new(red_client),
        http_client: Arc::new(http_client),
        db,

        idx_tx,
        meta_tx,
        url_tx,
        update_tx,

        multiplexed_acquire_config,
        pool_acquire_config,

        stream_config: stream_conf,
        onetime_config: onetime_conf,
        engine_config: engine_conf,
    });

    Router::new()
        .route("/create_meta", get(scraping_meta_process::<BasicRedisReq>))
        .route("/update_tag", get(update_handler::<BasicRedisReq>))
        .route("/roughs", get(find_roughs_handler))
        .route("/detail", get(find_detail_handler))
        .with_state(state)
}
