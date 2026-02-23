use std::{sync::Arc, time::Duration};

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, Client};
use tokio::sync::mpsc::Sender;

use crate::{
    redis_communication::RedisRequest,
    redis_window::{
        MultiplexedAcquireConfig, PoolAcquireConfig, RedisHandleErr, acquire::acquire_conn,
        redis_job::RedisJob,
    },
};

#[derive(Clone)]
pub struct OnetimeConfig {
    pub blocking_time: f64,
    pub how_long_to_wait: u64,
    pub hash_get_retry: usize,
    pub result_keyword: String
}

pub async fn onetime_req<RR>(
    url: &String,
    pool: Arc<Pool<RedisConnectionManager>>,
    pool_config: Arc<PoolAcquireConfig>,

    client: Arc<Client>,
    multi_config: Arc<MultiplexedAcquireConfig>,

    req_tx: Sender<String>,
    onetime_config: Arc<OnetimeConfig>,
) -> Result<String, RedisHandleErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let mut job = RedisJob::new();
    let req_string = {
        let req: RR = job.generate_redis_request(url.clone());
        serde_json::to_string(&req).unwrap()
    };

    if let Err(e) = req_tx.send(req_string).await {
        tracing::error!("{e}");
        return Err(RedisHandleErr::RequestSenderErr);
    };

    let mut conn;
    match client
        .get_multiplexed_async_connection_with_config(&multi_config.connection_config)
        .await
    {
        Ok(got_conn) => {
            conn = got_conn;
        }
        Err(e) => {
            tracing::error!("{e}");
            return Err(RedisHandleErr::RedisNoneConnection);
        }
    };

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(onetime_config.how_long_to_wait)) => {
            Err(RedisHandleErr::Timeout)
        },
        result = async move {
            let redis_job_id = job.get_id();
            loop {
                match conn.brpop::<&String, (String, String)>(&redis_job_id, onetime_config.blocking_time).await {
                    Ok(got_result) => {
                        let (_, task_id) = got_result;

                        let mut tempt = 0;
                        while tempt < onetime_config.hash_get_retry {
                            match acquire_conn(pool_config.clone(), &pool , None).await {
                                Some(mut got_conn) => {
                                    match got_conn.hget::<&String, &String, String> (&onetime_config.result_keyword, &task_id).await {
                                        Ok(hash_got) => {
                                            return Ok(hash_got);
                                        },
                                        Err(e) => {
                                            tracing::error!("{e}");
                                            tempt += 1;
                                        }
                                    };
                                },
                                None => {
                                    return Err(RedisHandleErr::RedisNoneConnection)
                                }
                            };
                        } // loop out

                        return Err(RedisHandleErr::OverRetry);
                    },
                    Err(e) => {
                        tracing::error!("{e}");
                    }
                }
            }
        } => {
            result
        }
    }
}
