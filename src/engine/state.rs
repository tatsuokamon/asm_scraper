use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::redis_window::{
    MultiplexedAcquireConfig, OnetimeConfig, PoolAcquireConfig, StreamConfig,
};

pub type EngineState = Arc<EngineStateStruct>;

pub struct EngineStateStruct {
    pub pool: Arc<Pool<RedisConnectionManager>>,
    pub red_client: Arc<redis::Client>,

    pub http_client: Arc<reqwest::Client>,
    pub db: sea_orm::DatabaseConnection,
    pub idx_tx: Sender<String>,
    pub meta_tx: Sender<String>,
    pub url_tx: Sender<String>,

    pub multiplexed_acquire_config: Arc<MultiplexedAcquireConfig>,
    pub pool_acquire_config: Arc<PoolAcquireConfig>,

    pub stream_config: Arc<StreamConfig>,
    pub onetime_config: Arc<OnetimeConfig>,
    pub engine_config: EngineConfig,
}

pub struct EngineConfig {
    pub count_channel_buf: usize,
    pub sse_channel_buf: usize,
}
