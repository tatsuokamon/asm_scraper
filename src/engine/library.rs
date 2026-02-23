use crate::{
    engine::err::EngineErr, redis_communication::RedisResponse, redis_window::RedisHandleErr,
};

pub fn parse_received<Parsed: serde::de::DeserializeOwned>(
    received: Result<String, RedisHandleErr>,
) -> Result<Parsed, EngineErr> {
    let result_string = received?;
    let resp = serde_json::from_str::<RedisResponse>(&result_string)
        .map_err(|_| EngineErr::InvalidReponseErr(result_string.clone()))?;
    match resp.error {
        Some(err) => Err(EngineErr::RedisServerErr(err)),
        None => match resp.payload {
            None => Err(EngineErr::InvalidReponseErr(result_string.clone())),
            Some(payload) => match serde_json::from_str::<Parsed>(&payload) {
                Ok(parsed) => Ok(parsed),
                Err(e) => {
                    tracing::error!("{e}");
                    Err(EngineErr::InvalidReponseErr(result_string.clone()))
                }
            },
        },
    }
}
