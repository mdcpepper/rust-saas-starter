use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_governor::GovernorError;

use super::errors::ApiError;

#[derive(Debug)]
pub struct RateLimitConfig {
    /// The number of requests allowed per second
    pub per_second: u64,

    /// The number of requests allowed in a burst
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            per_second: 2,
            burst_size: 5,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TooManyRequestsResponse {
    pub retry_after: u64,
}

/// Rate limit error handler
pub fn rate_limit_error_handler(err: GovernorError) -> Response<Body> {
    match err {
        GovernorError::TooManyRequests { wait_time, .. } => {
            let body = json!(TooManyRequestsResponse {
                retry_after: wait_time
            })
            .to_string();
            Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap()
        }
        _ => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            .into_response(),
    }
}
