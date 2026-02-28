use std::convert::Infallible;

use crate::{
    db_executor::create_meta,
    engine::{
        err::EngineErr,
        handlers::{finding_idx, finding_meta, finding_urls},
        library::parse_received,
        state::EngineState,
    },
    model::FindMetaResponse,
    redis_communication::{BasicRedisReq, RedisRequest},
};
use axum::{
    extract::{Query, State},
    response::{Sse, sse::Event},
};
use futures::{Stream, StreamExt, stream};
use tokio::{sync::mpsc::Sender, task::JoinSet};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

#[derive(serde::Deserialize)]
pub struct ProcessMetaQuery {
    target_url: String,
}

pub async fn finding_meta_process(
    State(stt): State<EngineState>,
    Query(q): Query<ProcessMetaQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static> {
    _finding_meta_process::<BasicRedisReq>(State(stt), Query(q)).await
}

async fn _finding_meta_process<RR>(
    State(stt): State<EngineState>,
    Query(q): Query<ProcessMetaQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send + 'static>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    // let mut main_set = JoinSet::new();

    let token = CancellationToken::new();
    let token_for_urls = token.child_token();
    let token_for_meta = token.child_token();
    let (count_tx, mut count_rx) = tokio::sync::mpsc::channel(stt.engine_config.count_channel_buf);
    let (sse_tx, sse_rx) =
        tokio::sync::mpsc::channel::<Result<i32, EngineErr>>(stt.engine_config.sse_channel_buf);

    tokio::spawn(async move {
        let mut back_set = JoinSet::new();
        tracing::info!("spawned _real_finding_meta process");
        if let Err(e) = _real_finding_meta_process::<RR>(
            &mut back_set,
            token_for_urls.clone(),
            token_for_meta,
            count_tx,
            q.target_url,
            State(stt.clone()),
        )
        .await
        {
            tracing::error!("{e}");
            token_for_urls.cancel();
            while back_set.join_next().await.is_some() {}
        };
        tracing::info!("spawned _real_finding_meta process: finish")
    });

    tokio::spawn(async move {
        tracing::info!("spawned count process");
        let mut count = 0;
        while let Some(result) = count_rx.recv().await {
            match result {
                Ok(_) => {
                    count += 1;
                    if count % 10 == 0 {
                        if let Err(e) = sse_tx.send(Ok(count)).await {
                            tracing::error!("{e}");
                        }
                    }
                }
                Err(e) => {
                    if let Err(e) = sse_tx.send(Err(e)).await {
                        tracing::error!("{e}");
                    }
                }
            }
        }
        if let Err(e) = sse_tx.send(Ok(count)).await {
            tracing::error!("{e}");
        }
        tracing::info!("spawned countfinished");
    });

    let main_stream = ReceiverStream::new(sse_rx).map(|result| {
        Ok(match result {
            Ok(count) => Event::default().data(format!("{}", count)),
            Err(e) => {
                tracing::error!("{}", &e);
                Event::default()
                    .event("error")
                    .data(format!("error while sse: {}", e))
            }
        })
    });

    Sse::new(main_stream.chain(stream::once(async {
        Ok(Event::default().event("end").data("complete"))
    })))
}

async fn _real_finding_meta_process<RR>(
    set: &mut JoinSet<()>,
    token_for_urls: CancellationToken,
    token_for_meta: CancellationToken,
    count_tx: Sender<Result<(), EngineErr>>,
    target_url: String,

    State(stt): State<EngineState>,
) -> Result<(), EngineErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let idx = finding_idx::<RR>(State(stt.clone()), &target_url).await?;

    let finding_urls_receiver =
        finding_urls::<RR>(set, token_for_urls, &target_url, idx, State(stt.clone())).await?;

    let mut finding_meta_reciever = finding_meta::<RR>(
        set,
        token_for_meta,
        finding_urls_receiver,
        Some(false),
        State(stt.clone()),
    )
    .await?;

    while let Some(received_meta_result) = finding_meta_reciever.recv().await {
        let sending_content = match parse_received::<FindMetaResponse>(received_meta_result) {
            Ok(parsed) => {
                tracing::info!("{}", &parsed.title);
                create_meta(parsed, &stt.db, stt.http_client.clone())
                    .await
                    .map_err(EngineErr::DBExecutorErr)
            }

            Err(e) => Err(e),
        };
        if let Err(e) = count_tx.send(sending_content).await {
            tracing::error!("{e}");
        }
    }

    Ok(())
}
