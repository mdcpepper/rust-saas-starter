//! Error handling for the API

use anyhow::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// An error response
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// The error message
    #[schema(example = "Internal server error")]
    pub error: String,
}

/// An error raised in the API
#[derive(Debug)]
pub struct ApiError {
    /// The status code
    pub status: StatusCode,

    /// The error message
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::usize;

    use anyhow::anyhow;
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
    use testresult::TestResult;

    use super::ApiError;

    #[tokio::test]
    async fn test_error_response() -> TestResult {
        let error = ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal server error".to_string(),
        };

        let response = error.into_response();
        let body = to_bytes(response.into_body(), usize::MAX).await?;

        assert_eq!(body, r#"{"error":"Internal server error"}"#);

        Ok(())
    }

    #[test]
    fn test_api_error_from_error() {
        let error = anyhow!("Internal server error");
        let api_error = ApiError::from(error);

        assert_eq!(api_error.status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(api_error.message, "Internal server error");
    }
}
