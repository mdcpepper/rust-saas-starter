//! API handler modules

use std::any::Any;

// use askama_axum::IntoResponse;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

use super::errors::ErrorResponse;

pub mod v1;

/// Catch panics and return a 500 error
pub fn panic_handler(err: Box<dyn Any + Send + 'static>) -> Response<Body> {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Internal server error".to_string()
    };

    let error = ErrorResponse { error: details };

    let response = Json(error).into_response();

    (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn test_panic_handler() {
        let panic_info = simulate_panic();
        let response = panic_handler(panic_info);

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_text = String::from_utf8(body.to_vec()).unwrap();

        let json = serde_json::from_str::<serde_json::Value>(&body_text).unwrap();

        assert_eq!(json, serde_json::json!({ "error": "Something went wrong" }));
    }

    fn simulate_panic() -> Box<dyn std::any::Any + Send + 'static> {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            panic!("Something went wrong");
        }));

        if let Err(err) = result {
            err
        } else {
            panic!("The panic did not occur as expected");
        }
    }
}
