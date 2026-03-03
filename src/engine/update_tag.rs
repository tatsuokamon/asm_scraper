use axum::{
    Json,
    extract::{Query, State},
};
use sea_orm::{DatabaseConnection, DbErr};
use std::str::FromStr;

use crate::{
    db_executor::{self, TagEntityExt}, engine::{
        AxumResponse, FindResultResponse, err::EngineErr, handlers::{UpdateSrc, update_tag}, state::EngineState
    }, entity::{circle, cv, genre, illust, scenario, series}, model::TagSrc, redis_communication::RedisRequest
};

#[derive(thiserror::Error, Debug)]
pub enum UpdateConvertErr {
    #[error("Not Found")]
    NotFound,
}

#[derive(thiserror::Error, Debug)]
pub enum UpdateTagErr {
    #[error("{0}")]
    UpdateConvert(#[from] UpdateConvertErr),

    #[error("{0}")]
    DBErr(#[from] DbErr),

    #[error("{0}")]
    Engine(#[from] EngineErr),
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
    match update_handler_inner::<RR>(&q.target, stt).await {
        Ok(content) => {
            (
                axum::http::StatusCode::OK,
                Json(UpdateResponse {
                    payload: Some(content),
                    error: None
                })
            )
        },
        Err(e) => {
            tracing::error!("{e}");
            match e {
                UpdateConvertErr => {
                    (
                        axum::http::StatusCode::BAD_REQUEST,
                        Json(UpdateResponse {
                            payload: None,
                            error: Some("".to_string())
                        })
                    )
                },
                _ => {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(UpdateResponse {
                            payload: None,
                            error: Some("".to_string())
                        })
                    )
                }
            }
        }
    }
}

async fn update_handler_inner<RR>(
    q: &str,
    stt: EngineState,
) -> Result<Vec<TagSrc>, UpdateTagErr>
where
    RR: RedisRequest + serde::Serialize,
{
    let src = UpdateSrc::from_str(q)?;
    let content = update_tag::<RR>(&src, State(stt.clone())).await?;
    match src {
        UpdateSrc::CV => {
            db_executor::update_tag::<cv::Entity>(&content, &stt.db).await?;
        },
        UpdateSrc::Series=> {
            db_executor::update_tag::<series::Entity>(&content, &stt.db).await?;
        },
        UpdateSrc::Genre=> {
            db_executor::update_tag::<genre::Entity>(&content, &stt.db).await?;
        },
        UpdateSrc::Circle=> {
            db_executor::update_tag::<circle::Entity>(&content, &stt.db).await?;
        },
        UpdateSrc::Illust=> {
            db_executor::update_tag::<illust::Entity>(&content, &stt.db).await?;
        },
        UpdateSrc::Scenario=> {
            db_executor::update_tag::<scenario::Entity>(&content, &stt.db).await?;
        },
    }

    Ok(content)
}
