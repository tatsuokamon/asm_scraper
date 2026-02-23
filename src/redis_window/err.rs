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

#[derive(thiserror::Error, Debug)]
pub enum RedisHandleErr {
    #[error("RedisHandleErr Timeout")]
    Timeout,

    #[error("RedisHandleErr ConnectionDrop")]
    ConnectionDrop,

    #[error("RedisHandleErr RedisNoneConnection")]
    RedisNoneConnection,

    #[error("RedisHandleErr RequestSenderErr")]
    RequestSenderErr,

    #[error("RedisHandleErr IDSenderErr")]
    IDSenderErr,

    #[error("RedisHandleErr OverRetry")]
    OverRetry,
}
