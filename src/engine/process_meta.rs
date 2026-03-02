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
    redis_communication::RedisRequest,
};
use axum::{
    extract::{Query, State},
    response::{Sse, sse::Event},
};
use futures::{Stream, StreamExt, stream};
use tokio::{sync::mpsc::Sender, task::JoinSet};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

pub enum ScrapeMetaSrc<'a> {
    CV(&'a str),
    Circle(&'a str),
    Scenario(&'a str),
    Illust(&'a str),
    Series(&'a str),
    Genre(&'a str),
}

#[derive(thiserror::Error, Debug)]
pub enum ScrapeConvertErr {
    #[error("Not found")]
    NotFound,
}

impl<'a> ScrapeMetaSrc<'a> {
    fn from_str_to_src<'b>(
        s: &'b str,
        content: &'b str,
    ) -> Result<ScrapeMetaSrc<'b>, ScrapeConvertErr> {
        match s {
            "cv" => Ok(ScrapeMetaSrc::CV(content)),
            "circle" => Ok(ScrapeMetaSrc::Circle(content)),
            "scenario" => Ok(ScrapeMetaSrc::Scenario(content)),
            "illust" => Ok(ScrapeMetaSrc::Illust(content)),
            "series" => Ok(ScrapeMetaSrc::Series(content)),
            "genre" => Ok(ScrapeMetaSrc::Genre(content)),
            _ => Err(ScrapeConvertErr::NotFound),
        }
    }

    fn to_url(&self) -> String {
        let url_template = "https://asmr18.fans/";
        match self {
            Self::CV(content) => format!("{}cv/{}/", url_template, content),
            Self::Circle(content) => format!("{}circle/{}/", url_template, content),
            Self::Scenario(content) => format!("{}scenario/{}/", url_template, content),
            Self::Illust(content) => format!("{}illust/{}/", url_template, content),
            Self::Series(content) => format!("{}series/{}/", url_template, content),
            Self::Genre(content) => format!("{}genre/{}/", url_template, content),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ProcessMetaQuery {
    pub kind: String,
    pub value: String,
}

pub async fn scraping_meta_process<RR>(
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
        if let Err(e) = _real_scraping_meta_process::<RR>(
            &mut back_set,
            token_for_urls.clone(),
            token_for_meta,
            count_tx,
            q,
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

async fn _real_scraping_meta_process<RR>(
    set: &mut JoinSet<()>,
    token_for_urls: CancellationToken,
    token_for_meta: CancellationToken,
    count_tx: Sender<Result<(), EngineErr>>,
    q: ProcessMetaQuery,

    State(stt): State<EngineState>,
) -> Result<(), EngineErr>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let target_url = ScrapeMetaSrc::from_str_to_src(&q.kind, &q.value)?.to_url();
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
