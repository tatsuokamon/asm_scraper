use serde::{Deserialize, Serialize};

pub trait RedisRequest<'a> {
    fn get_url(&self) -> String;
    fn get_id(&self) -> String;
    fn get_job_id(&self) -> String;
    fn index(&self) -> i32;
    fn new(url: String, job_id: &'a str, index: i32) -> Self;
}

#[derive(Serialize)]
pub struct BasicRedisReq<'a> {
    pub url: String,
    id: String,
    pub job_id: &'a str,
    pub index: i32,
}

#[derive(Deserialize)]
pub struct RedisResponse {
    pub error: Option<String>,
    pub payload: Option<String>,
    pub index: i32,
}

impl<'a> RedisRequest<'a> for BasicRedisReq<'a> {
    fn new(url: String, job_id: &'a str, index: i32) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            url,
            job_id,
            index,
            id,
        }
    }

    fn get_url(&self) -> String {
        self.url.clone()
    }
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_job_id(&self) -> String {
        self.job_id.clone()
    }

    fn index(&self) -> i32 {
        self.index
    }
}
