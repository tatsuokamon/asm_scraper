mod err;
mod library;
mod state;

mod handlers;

mod find_detail;
mod find_roughs;
mod process_meta;
mod update_tag;

mod router;

use axum::Json;
pub use router::ready_router;
pub use state::EngineConfig;

type AxumResponse<Content: serde::Serialize> = (axum::http::StatusCode, Json<Content>);

#[derive(serde::Serialize)]
pub struct FindResultResponse<T: serde::Serialize> {
    pub payload: Option<T>,
    pub error: Option<String>,
}
