use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};

use crate::{
    db_executor::{Roughs, TagFinderStruct, find_roughs},
    engine::{AxumResponse, FindResultResponse, state::EngineState},
};

type FindRoughsResponse = FindResultResponse<Roughs>;

#[derive(serde::Deserialize)]
pub struct FindRoughsQuery {
    pub index: Option<i32>,
    pub size: Option<i32>,

    pub cvs: Option<Vec<String>>,
    pub illusts: Option<Vec<String>>,
    pub series: Option<Vec<String>>,
    pub circles: Option<Vec<String>>,
    pub genres: Option<Vec<String>>,
}

impl From<FindRoughsQuery> for TagFinderStruct {
    fn from(value: FindRoughsQuery) -> Self {
        Self {
            index: value.index,
            size: value.size,
            cvs: value.cvs,
            illusts: value.illusts,
            series: value.series,
            circles: value.circles,
            genres: value.genres,
        }
    }
}

pub async fn find_roughs_handler(
    State(stt): State<EngineState>,
    Query(q): Query<FindRoughsQuery>,
) -> AxumResponse<FindRoughsResponse> {
    match find_roughs(q.into(), &stt.db).await {
        Ok(roughs) => (
            StatusCode::OK,
            Json(FindRoughsResponse {
                payload: Some(roughs),
                error: None,
            }),
        ),
        Err(e) => {
            tracing::error!("{e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FindRoughsResponse {
                    payload: None,
                    error: Some("".to_string()),
                }),
            )
        }
    }
}
