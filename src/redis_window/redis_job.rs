use crate::redis_communication::RedisRequest;
use bb8_redis::RedisConnectionManager;

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

    pub fn generate_redis_request<RR: RedisRequest>(
        &mut self,
        url: String,
        force: Option<bool>,
    ) -> RR {
        self.idx += 1;
        RR::new(url, self.id.clone(), self.idx, force)
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
