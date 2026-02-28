use redis::AsyncCommands;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::Sender;

use crate::redis_window::{
    acquire::{MultiplexedAcquireConfig, acquire_conn},
    err::RedisHandleErr,
    stream::StreamState
};

pub async fn create_stream_id_getter_part(
    result_tx: Sender<Result<String, RedisHandleErr>>,
    redis_job_id: String,
    id_tx: Sender<String>,

    client: Arc<redis::Client>,
    client_acquire_config: Arc<MultiplexedAcquireConfig>,

    stream_state: Arc<StreamState>,

    blocking_time: f64,
    how_long_to_wait: u64,
) -> () {
    let result_tx_for_sleep = result_tx.clone();
    let result_tx_for_main = result_tx;

    let stream_state_moved_to_sleep = stream_state.clone();
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(how_long_to_wait)) => {
            stream_state_moved_to_sleep.abort();
            if let Err(e) = result_tx_for_sleep.send(Err(RedisHandleErr::Timeout)).await {
                tracing::error!("{e}");
            };
        },
        _ = async move {
            let mut conn;

            match acquire_conn(client_acquire_config.clone(), &client, None).await {
                Some(got_conn) => {
                    conn = got_conn;
                },
                None => {
                    if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::RedisNoneConnection)).await {
                        tracing::error!("{e}");
                    };
                    return;
                }
            };

            loop {
                if stream_state.finished() {
                    break;
                }

                match conn.brpop::<&String, (String, String)>(&redis_job_id, blocking_time).await {
                    Ok(received) => {
                        stream_state.inflight_sub(1); // idが戻ってきた時点でstate側の終了条件とする (完全にresultを返す時点ではないことに注意)
                        let (_, task_id) = received;
                        if let Err(e) = id_tx.send(task_id).await {
                            tracing::error!("{e}");
                            if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::IDSenderErr)).await {
                                tracing::error!("{e}");
                                continue;
                            }
                        }

                    },
                    Err(e) => {
                        tracing::error!("{e}");
                        match acquire_conn(client_acquire_config.clone(), &client, None).await {
                            Some(c) => {
                                conn = c;
                            },
                            None => {
                                if let Err(e) = result_tx_for_main.send(Err(RedisHandleErr::RedisNoneConnection)).await {
                                    tracing::error!("{e}");
                                }
                            }
                        }
                    }
                }
            } // loop out
        } => {}
    }
}
