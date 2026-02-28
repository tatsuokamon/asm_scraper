use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    redis_communication::RedisRequest,
    redis_window::{err::RedisHandleErr, redis_job::RedisJob, stream::StreamState},
};

pub struct RequestContract {
    pub req_tx: Sender<String>,
    pub force: Option<bool>,
}

pub async fn create_stream_request_part<RR>(
    mut url_rx: Receiver<String>,
    result_tx: Sender<Result<String, RedisHandleErr>>,

    mut redis_job: RedisJob,
    stream_state: Arc<StreamState>,
    req_contract: RequestContract,
) -> ()
where
    RR: RedisRequest + serde::ser::Serialize,
{
    while let Some(url) = url_rx.recv().await {
        let req_string = {
            let redis_req: RR = redis_job.generate_redis_request(url.clone(), req_contract.force);
            serde_json::to_string(&redis_req).unwrap()
        };

        if let Err(e) = req_contract.req_tx.send(req_string).await {
            tracing::error!("{e}");
            if let Err(e) = result_tx.send(Err(RedisHandleErr::RequestSenderErr)).await {
                tracing::error!("{e}");
            };
            continue;
        }

        stream_state.inflight_add(1);
    } // loop out
    //

    stream_state.store_finished_state(true);
}
