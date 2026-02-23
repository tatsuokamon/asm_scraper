#[derive(thiserror::Error, Debug)]
pub enum DBExecutorErr {
    #[error("DBErr {0}")]
    DBErr(#[from] sea_orm::DbErr),

    #[error("ReqwestErr {0}")]
    ReqwestErr(#[from] reqwest::Error),
}
