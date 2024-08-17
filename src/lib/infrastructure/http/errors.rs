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
