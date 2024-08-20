//! API error-handling module

use std::fmt;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::{
    auth::users::{
        errors::{CreateUserError, EmailConfirmationError, GetUserByIdError},
        PasswordError,
    },
    communication::email_addresses::EmailAddressError,
};

/// An error response
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// The error message
    #[schema(example = "Internal server error")]
    pub error: String,
}

/// An error raised in the API
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApiError {
    /// The status code
    #[schema(example = 500, value_type = u16)]
    #[serde(with = "http_serde::status_code")]
    pub status: StatusCode,

    /// The error message
    #[schema(example = "Internal server error")]
    pub message: String,
}

impl ApiError {
    /// Create a new API error
    pub fn new(status: StatusCode, message: &str) -> Self {
        Self {
            status,
            message: message.to_string(),
        }
    }

    pub fn new_404(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn new_409(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.to_string(),
        }
    }

    /// Create a new unprocessable entity error
    pub fn new_422(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.to_string(),
        }
    }

    /// Create new internal server error
    pub fn new_500(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
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

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

impl From<EmailAddressError> for ApiError {
    fn from(err: EmailAddressError) -> Self {
        match err {
            EmailAddressError::EmptyEmailAddress => {
                ApiError::new_422("Please provide an email address")
            }
            EmailAddressError::InvalidEmailAddress => {
                ApiError::new_422("Please provide a valid email address")
            }
        }
    }
}

impl From<EmailConfirmationError> for ApiError {
    fn from(err: EmailConfirmationError) -> Self {
        match err {
            EmailConfirmationError::UserNotFound(id) => {
                ApiError::new_404(&format!("User with id \"{id}\" not found"))
            }
            EmailConfirmationError::CouldNotSendEmail => {
                ApiError::new_500("Could not send email confirmation email")
            }
            EmailConfirmationError::EmailAlreadyConfirmed => {
                ApiError::new_409("Email is already confirmed")
            }
            EmailConfirmationError::ConfirmationTokenExpired => {
                ApiError::new_422("Confirmation token has expired")
            }
            EmailConfirmationError::ConfirmationTokenMismatch => {
                ApiError::new_422("Confirmation token does not match")
            }
            EmailConfirmationError::UnknownError(err) => unknown_error(Some(err.to_string())),
        }
    }
}

impl From<PasswordError> for ApiError {
    fn from(err: PasswordError) -> Self {
        match err {
            PasswordError::TooShort => {
                ApiError::new_422("Password must be at least 8 characters long")
            }
            PasswordError::TooLong => {
                ApiError::new_422("Password must be at most 100 characters long")
            }
            PasswordError::TooWeak(suggestions) => {
                ApiError::new_422(&format!("Password is too weak: {}", suggestions.join(" ")))
            }
        }
    }
}

impl From<CreateUserError> for ApiError {
    fn from(err: CreateUserError) -> Self {
        match err {
            CreateUserError::DuplicateUser { email } => {
                ApiError::new_409(&format!("User with email \"{email}\" already exists"))
            }
            CreateUserError::UnknownError(err) => unknown_error(Some(err.to_string())),
        }
    }
}

impl From<GetUserByIdError> for ApiError {
    fn from(err: GetUserByIdError) -> Self {
        match err {
            GetUserByIdError::UserNotFound(id) => {
                ApiError::new_404(&format!("User with id \"{id}\" not found"))
            }
            GetUserByIdError::UnknownError(err) => unknown_error(Some(err.to_string())),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        ApiError::new(rejection.status(), &rejection.body_text())
    }
}

fn unknown_error(message: Option<String>) -> ApiError {
    // TODO: Just log the message and return the generic one
    if let Some(message) = message {
        ApiError::new_500(&message)
    } else {
        ApiError::new_500("An unknown error occurred, please try again")
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
