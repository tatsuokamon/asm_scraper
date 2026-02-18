use std::sync::Arc;

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};

use crate::redis_communication::RedisRequest;

pub struct RedisJob {
    id: String,
    idx: i32,
}

// jobを意識せずに保持したい！！
impl RedisJob {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            idx: 0,
        }
    }

    pub fn generate_redis_request<'a, RR: RedisRequest<'a>>(&'a mut self, url: String) -> RR {
        RR::new(url, &self.id, self.idx)
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
