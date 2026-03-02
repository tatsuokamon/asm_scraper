use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    db_executor::{Detail, find_meta_with_id},
    engine::{AxumResponse, FindResultResponse, find_detail, state::EngineState},
};

type FindDetailResponse = FindResultResponse<Detail>;

#[derive(serde::Deserialize)]
pub struct FindDetailQuery {
    pub id: String,
}

pub async fn find_detail_handler(
    State(stt): State<EngineState>,
    Query(q): Query<FindDetailQuery>,
) -> AxumResponse<FindDetailResponse> {
    match find_meta_with_id(q.id, &stt.db).await {
        Ok(Some(result)) => (
            axum::http::StatusCode::OK,
            Json(FindDetailResponse {
                payload: Some(result),
                error: None,
            }),
        ),
        Ok(None) => (
            axum::http::StatusCode::OK,
            Json(FindDetailResponse {
                payload: None,
                error: Some("not found".to_string()),
            }),
        ),
        Err(e) => {
            tracing::error!("{e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(FindDetailResponse {
                    payload: None,
                    error: Some("".to_string()),
                }),
            )
        }
    }
}
