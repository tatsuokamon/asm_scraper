use std::{fmt::Debug, pin::Pin, sync::Arc, time::Duration};

use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncConnectionConfig, Client, RedisError, aio::MultiplexedConnection};
use tokio_util::sync::CancellationToken;

pub trait AcquireConfigTrait<Source, E>: Send + Sync
where
    Source: Send + Sync,
    E: Send + Sync,
{
    type Output<'a>
    where
        Source: 'a;

    fn get_retry(&self) -> i32;
    fn get_backoff_init(&self) -> Duration;
    fn next_backoff(&self, current_duration: Duration) -> Duration;
    async fn acquire<'b>(&self, src: &'b Source) -> Result<Self::Output<'b>, E>;
}

pub struct MultiplexedAcquireConfig {
    pub retry: i32,
    pub connection_config: AsyncConnectionConfig,
    pub backoff_init: Duration,
    pub backoff_algo: Arc<dyn Fn(Duration) -> Duration + Send + Sync + 'static>,
}

pub struct PoolAcquireConfig {
    pub retry: i32,
    pub backoff_init: Duration,
    pub backoff_algo: Arc<dyn Fn(Duration) -> Duration + Send + Sync + 'static>,
}

impl AcquireConfigTrait<Arc<Client>, RedisError> for MultiplexedAcquireConfig {
    type Output<'a> = MultiplexedConnection;

    fn get_retry(&self) -> i32 {
        self.retry
    }

    fn get_backoff_init(&self) -> Duration {
        self.backoff_init.clone()
    }

    fn next_backoff(&self, current_duration: Duration) -> Duration {
        (self.backoff_algo)(current_duration)
    }

    async fn acquire<'b>(&self, src: &'b Arc<Client>) -> Result<Self::Output<'b>, RedisError> {
        src.get_multiplexed_async_connection_with_config(&self.connection_config)
            .await
    }
}

impl AcquireConfigTrait<Arc<Pool<RedisConnectionManager>>, bb8::RunError<RedisError>>
    for PoolAcquireConfig
{
    type Output<'a> = PooledConnection<'a, RedisConnectionManager>;

    fn get_retry(&self) -> i32 {
        self.retry
    }

    fn get_backoff_init(&self) -> Duration {
        self.backoff_init.clone()
    }

    fn next_backoff(&self, current_duration: Duration) -> Duration {
        (self.backoff_algo)(current_duration)
    }

    async fn acquire<'b>(
        &self,
        src: &'b Arc<Pool<RedisConnectionManager>>,
    ) -> Result<Self::Output<'b>, bb8::RunError<RedisError>> {
        src.get().await
    }
}

async fn connection_coroutine<'a, Source, E, Config>(
    cfg: &'a Config,
    src: &'a Source,
) -> Option<Config::Output<'a>>
where
    Config: AcquireConfigTrait<Source, E>,
    E: Debug + Send + Sync,
    Source: Send + Sync,
    Config::Output<'a>: Send + Sync,
{
    let mut tempt = 0;
    let mut backoff = cfg.get_backoff_init().clone();
    let retry = cfg.get_retry();

    while tempt < retry {
        match cfg.acquire(src).await {
            Ok(con) => return Some(con),
            Err(e) => {
                tracing::error!("{:?}", e);
                tokio::time::sleep(backoff).await;
                backoff = cfg.next_backoff(backoff);
                tempt += 1;
            }
        }
    }

    return None;
}

pub async fn acquire_conn<'a, Source, E: Debug, Config>(
    cfg: &'a Config,
    src: &'a Source,
    token_op: Option<&CancellationToken>,
) -> Option<Config::Output<'a>>
where
    Config: AcquireConfigTrait<Source, E>,
    E: Debug + Send + Sync,
    Source: Send + Sync,
    Config::Output<'a>: Send + Sync,
{
    match token_op {
        Some(token) => {
            tokio::select! {
                result = connection_coroutine(cfg, src) => {
                    result
                },
                _ = token.cancelled() => {
                    None
                }
            }
        }
        None => connection_coroutine(cfg, src).await,
    }
}
