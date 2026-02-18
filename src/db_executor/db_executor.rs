pub struct DBExecutor {
    db: sea_orm::DatabaseConnection,
    client: reqwest::Client,
}

impl DBExecutor {
    pub async fn insert_meta_src() {}
}
