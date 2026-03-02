use crate::{
    db_executor::DBExecutorErr,
    engine::process_meta::ScrapeConvertErr,
    redis_window::{self, RedisHandleErr},
};

#[derive(thiserror::Error, Debug)]
pub enum EngineErr {
    #[error("{0}")]
    RedisWindowErr(#[from] redis_window::RedisWindowErr),

    #[error("{0}")]
    RedisCommunicationErr(String),

    #[error("{0}")]
    RedisHandleErr(#[from] RedisHandleErr),

    #[error("EngineErr ResponseNoneErr")]
    ResponseNoneErr,

    #[error("EngineErr InvalidReponseErr")]
    InvalidReponseErr(String),

    #[error("EngineErr RedisServerErr {0}")]
    RedisServerErr(String),

    #[error("EngineErr DBExecutorErr {0}")]
    DBExecutorErr(#[from] DBExecutorErr),

    #[error("EngineErr ScrapeConvertErr {0}")]
    ScrapeConvertErr(#[from] ScrapeConvertErr),
}
