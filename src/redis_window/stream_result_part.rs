use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::redis_window::{
    acquire::{PoolAcquireConfig, acquire_conn},
    err::RedisHandleErr,
};

pub async fn create_stream_result_part(
    result_tx: Sender<Result<String, RedisHandleErr>>,
    result_keyword: &String,
    mut id_rx: Receiver<String>,

    pool: Arc<Pool<RedisConnectionManager>>,
    pool_acquire_config: Arc<PoolAcquireConfig>,
    hget_retry: usize,
) -> () {
    let mut conn;
    match acquire_conn(pool_acquire_config.clone(), &pool, None).await {
        Some(c) => {
            conn = c;
        }
        None => {
            if let Err(e) = result_tx
                .send(Err(RedisHandleErr::RedisNoneConnection))
                .await
            {
                tracing::error!("{e}");
            };
            return;
        }
    };

    while let Some(received_id) = id_rx.recv().await {
        let mut tempt = 0;
        while tempt < hget_retry {
            match conn
                .hget::<&String, &String, String>(result_keyword, &received_id)
                .await
            {
                Ok(hash_got) => {
                    if let Err(e) = result_tx.send(Ok(hash_got)).await {
                        tracing::error!("{e}");
                    }
                    if let Err(e) = conn
                        .hdel::<&String, &String, i32>(result_keyword, &received_id)
                        .await
                    {
                        tracing::error!("{e}");
                    }
                }
                Err(e) => {
                    tracing::error!("{e}");

                    match acquire_conn(pool_acquire_config.clone(), &pool, None).await {
                        Some(c) => {
                            conn = c;
                        }
                        None => {
                            if let Err(e) = result_tx
                                .send(Err(RedisHandleErr::RedisNoneConnection))
                                .await
                            {
                                tracing::error!("{e}");
                            };
                            return;
                        }
                    };
                    tempt += 1;
                }
            }
        }
    }
}
