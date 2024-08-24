//! Error types for users, authentication and authorization

use anyhow::anyhow;
use thiserror::Error;
use tracing::debug;

/// Errors that can occur when creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    /// User with email already exists
    #[error("User already exists with that email address")]
    DuplicateUser,

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

/// Errors that can occur when getting a user
#[derive(Debug, Error)]
pub enum GetUserByIdError {
    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

/// Errors that can occur when updating a user
#[derive(Debug, Error)]
pub enum UpdateUserError {
    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Email address already in use
    #[error("User's email is already in use")]
    EmailAddressInUse,

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

impl From<sqlx::Error> for CreateUserError {
    fn from(err: sqlx::Error) -> Self {
        debug!("sqlxError: {:?}", err);

        match err {
            sqlx::Error::Database(db_err) => match db_err.kind() {
                sqlx::error::ErrorKind::UniqueViolation => CreateUserError::DuplicateUser,
                _ => CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", db_err)),
            },
            _ => CreateUserError::UnknownError(anyhow!("Unknown database error: {:?}", err)),
        }
    }
}

impl From<sqlx::Error> for UpdateUserError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UpdateUserError::UserNotFound,
            _ => UpdateUserError::UnknownError(err.into()),
        }
    }
}
