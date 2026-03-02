use axum::{
    Json,
    extract::{Query, State},
};
use std::str::FromStr;

use crate::{
    engine::{
        AxumResponse, FindResultResponse,
        handlers::{UpdateSrc, update_tag},
        state::EngineState,
    },
    model::TagSrc,
    redis_communication::RedisRequest,
};

#[derive(thiserror::Error, Debug)]
pub enum UpdateConvertErr {
    #[error("Not Found")]
    NotFound,
}

impl FromStr for UpdateSrc {
    type Err = UpdateConvertErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cv" => Ok(Self::CV),
            "circle" => Ok(Self::Circle),
            "scenario" => Ok(Self::Scenario),
            "illust" => Ok(Self::Illust),
            "series" => Ok(Self::Series),
            "genre" => Ok(Self::Genre),
            // "music" => Self::Music.ok, TODO:
            _ => Err(self::UpdateConvertErr::NotFound),
        }
    }
}
type UpdateResponse = FindResultResponse<Vec<TagSrc>>;

#[derive(serde::Deserialize)]
pub struct UpdateQuery {
    pub target: String,
}

pub async fn update_handler<RR>(
    State(stt): State<EngineState>,
    Query(q): Query<UpdateQuery>,
) -> AxumResponse<UpdateResponse>
where
    RR: RedisRequest + serde::ser::Serialize,
{
    let src;
    match UpdateSrc::from_str(&q.target) {
        Ok(s) => src = s,
        Err(e) => {
            tracing::error!("{e}");
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(UpdateResponse {
                    payload: None,
                    error: Some("no such target".to_string()),
                }),
            );
        }
    };
    match update_tag::<RR>(src, State(stt)).await {
        Ok(content) => (
            axum::http::StatusCode::OK,
            Json(UpdateResponse {
                payload: Some(content),
                error: None,
            }),
        ),
        Err(e) => {
            tracing::error!("{e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(UpdateResponse {
                    payload: None,
                    error: Some("error while processing".to_string()),
                }),
            )
        }
    }
}
