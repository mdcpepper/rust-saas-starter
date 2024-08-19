//! Error types for users, authentication and authorization

use thiserror::Error;
use uuid::Uuid;

use crate::domain::comms::value_objects::email_address::EmailAddress;

/// Errors that can occur when creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    /// User with email already exists
    #[error("user with email {email} already exists")]
    DuplicateUser {
        /// Email address
        email: EmailAddress,
    },

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

/// Errors that can occur when getting a user
#[derive(Debug, Error)]
pub enum GetUserByIdError {
    /// User not found
    #[error("user with id \"{0}\" not found")]
    UserNotFound(Uuid),

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}
