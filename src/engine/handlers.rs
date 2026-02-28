use axum::extract::State;
use tokio::{sync::mpsc::Receiver, task::JoinSet};
use tokio_util::sync::CancellationToken;

use crate::{
    engine::{err::EngineErr, library::parse_received, state::EngineState},
    model::FindDetailResponse,
    redis_communication::{RedisRequest, RedisResponse},
    redis_window::{RedisHandleErr, RequestContract, create_stream, onetime_req},
};

pub async fn finding_idx<RR>(
    State(stt): State<EngineState>,
    target_url_src: &String,
) -> Result<i32, EngineErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let got_idx_resp = onetime_req::<RR>(
        target_url_src,
        stt.pool.clone(),
        stt.pool_acquire_config.clone(),
        stt.red_client.clone(),
        stt.multiplexed_acquire_config.clone(),
        stt.idx_tx.clone(),
        stt.onetime_config.clone(),
    )
    .await?;

    match serde_json::from_str::<RedisResponse>(&got_idx_resp) {
        Ok(deserialized) => match deserialized.error {
            Some(er) => Err(EngineErr::RedisCommunicationErr(er)),
            None => match deserialized.payload {
                Some(payload) => Ok(payload
                    .parse::<i32>()
                    .map_err(|_| EngineErr::InvalidReponseErr(got_idx_resp.clone()))?),
                None => Err(EngineErr::ResponseNoneErr),
            },
        },
        Err(e) => {
            tracing::error!("{e}");
            Err(EngineErr::InvalidReponseErr(got_idx_resp.clone()))
        }
    }
}

pub async fn finding_urls<RR>(
    set: &mut JoinSet<()>,
    token: CancellationToken,
    target_url_src: &str,
    idx: i32,
    State(stt): State<EngineState>,
) -> Result<Receiver<Result<String, RedisHandleErr>>, EngineErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let (url_tx, result_rx) = create_stream::<RR>(
        set,
        token,
        stt.pool.clone(),
        stt.red_client.clone(),
        stt.multiplexed_acquire_config.clone(),
        stt.pool_acquire_config.clone(),
        stt.stream_config.clone(),
        RequestContract {
            req_tx: stt.url_tx.clone(),
            force: Some(true)
        }
    )
    .await?;

    let cloned_url = target_url_src.to_string();
    set.spawn(async move {
        let mut tempt = 0;

        while tempt < idx {
            let url = if tempt == 0 {
                cloned_url.clone()
            } else {
                format!("{}/page/{}", &cloned_url, tempt + 1)
            };

            if let Err(e) = url_tx.send(url).await {
                tracing::error!("{e}");
            }

            tempt += 1;
        }
    });

    Ok(result_rx)
}

pub async fn finding_meta<RR>(
    set: &mut JoinSet<()>,
    token: CancellationToken,

    mut url_response_rx: Receiver<Result<String, RedisHandleErr>>,
    force: Option<bool>,
    State(stt): State<EngineState>,
) -> Result<Receiver<Result<String, RedisHandleErr>>, EngineErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let (url_tx, result_rx) = create_stream::<RR>(
        set,
        token,
        stt.pool.clone(),
        stt.red_client.clone(),
        stt.multiplexed_acquire_config.clone(),
        stt.pool_acquire_config.clone(),
        stt.stream_config.clone(),
        RequestContract { req_tx: stt.meta_tx.clone(), force }
    )
    .await?;

    set.spawn(async move {
        while let Some(received) = url_response_rx.recv().await {
            match parse_received::<FindDetailResponse>(received) {
                Ok(resp) => {
                    for u in resp {
                        if let Err(e) = url_tx.send(u).await {
                            tracing::error!("{e}");
                        };
                    }
                }
                Err(e) => {
                    tracing::error!("{e}");
                    continue;
                }
            }
        }
    });

    Ok(result_rx)
}
