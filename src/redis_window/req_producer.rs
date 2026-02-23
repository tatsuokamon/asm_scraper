use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::redis_window::acquire::{PoolAcquireConfig, acquire_conn};
use crate::redis_window::err::RedisWindowErr;

pub async fn redis_request_producer(
    req_keyword: String,
    set: &mut JoinSet<()>,
    token: CancellationToken,
    pool: Arc<Pool<RedisConnectionManager>>,
    cfg: Arc<PoolAcquireConfig>,
    channel_buf: usize,
) -> Result<Sender<String>, RedisWindowErr> {
    pool.clone().get().await?;
    let (req_tx, mut req_rx) = tokio::sync::mpsc::channel(channel_buf);

    set.spawn(async move {
        let pool = pool;
        let mut conn;
        match acquire_conn(cfg.clone(), &pool, Some(&token)).await {
            Some(c) => {
                conn = c;
            }
            None => {
                tracing::error!("failed to get first conn");
                return;
            }
        };

        loop {
            tokio::select! {
                received = req_rx.recv() => {
                    if let Some(req) = received {
                        match push_with_retry(&req, &token, &mut conn, &pool, cfg.clone(), &req_keyword).await {
                            Ok(_) => {},
                            Err(e) =>  {
                                tracing::error!("sending req failed: {req}: {e}")
                            }
                        };
                    };
                },
                _ = token.cancelled() => {tracing::error!("loop out");break;}
            }
        }
    });

    Ok(req_tx)
}

async fn push_with_retry<'a>(
    req: &String,
    token: &CancellationToken,
    conn: &mut PooledConnection<'a, RedisConnectionManager>,
    pool: &'a Arc<Pool<RedisConnectionManager>>,
    cfg: Arc<PoolAcquireConfig>,
    req_keyword: &String,
) -> Result<(), RedisWindowErr> {
    let mut tempt = 0;
    while tempt < cfg.retry {
        loop {
            tokio::select! {
                push_result = conn.lpush::<&String, &String, ()>(req_keyword, req) => {
                    match push_result {
                        Ok(_) => {
                            return Ok(());
                        },
                        Err(e) => {
                            tracing::error!("{}", e);
                            match acquire_conn(cfg.clone(), pool, Some(token)).await {
                                Some(c) => {
                                    *conn = c;
                                },
                                None => {
                                    break;
                                }
                            };
                            tempt += 1;
                        }
                    }
                },
                _ = token.cancelled() => {
                },
            }
        }
    }

    return Err(RedisWindowErr::RequestLoopOutErr);
}
