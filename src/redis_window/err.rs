use bb8::RunError;
use redis::RedisError;

#[derive(thiserror::Error, Debug)]
pub enum RedisWindowErr {
    #[error("{0}")]
    RedisErr(#[from] RedisError),

    #[error("{0}")]
    RunErr(#[from] RunError<RedisError>),

    #[error("Request Loop out err")]
    RequestLoopOutErr,
}

#[derive(Debug)]
pub enum RedisHandleErr {
    Timeout,
    ConnectionDrop,
    RedisNoneConnection,
    RequestSenderErr,
    IDSenderErr,
}
