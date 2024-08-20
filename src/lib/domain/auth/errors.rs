//! Error types for users, authentication and authorization

use thiserror::Error;
use uuid::Uuid;

use crate::domain::communication::{
    errors::EmailError, value_objects::email_address::EmailAddress,
};

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

/// Errors that can occur when sending or verifying an email confirmation
#[derive(Debug, Error)]
pub enum EmailConfirmationError {
    /// User not found
    #[error("user with id \"{0}\" not found")]
    UserNotFound(Uuid),

    /// Could not send confirmation email
    #[error("could not send confirmation email")]
    CouldNotSendEmail,

    /// Email is already confirmed
    #[error("email is already confirmed")]
    EmailAlreadyConfirmed,

    /// Confirmation token expired
    #[error("confirmation token expired")]
    ConfirmationTokenExpired,

    /// Confirmation token mismatch
    #[error("confirmation token mismatch")]
    ConfirmationTokenMismatch,

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

/// Errors that can occur when updating a user
#[derive(Debug, Error)]
pub enum UpdateUserError {
    /// User not found
    #[error("user with id \"{0}\" not found")]
    UserNotFound(Uuid),

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

impl From<GetUserByIdError> for EmailConfirmationError {
    fn from(err: GetUserByIdError) -> Self {
        match err {
            GetUserByIdError::UserNotFound(id) => EmailConfirmationError::UserNotFound(id),
            GetUserByIdError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
        }
    }
}

impl From<UpdateUserError> for EmailConfirmationError {
    fn from(err: UpdateUserError) -> Self {
        match err {
            UpdateUserError::UserNotFound(id) => EmailConfirmationError::UserNotFound(id),
            UpdateUserError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
        }
    }
}

impl From<EmailError> for EmailConfirmationError {
    fn from(err: EmailError) -> Self {
        match err {
            EmailError::SendError | EmailError::InvalidEmail => {
                EmailConfirmationError::CouldNotSendEmail
            }
            EmailError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
        }
    }
}

impl From<askama::Error> for EmailConfirmationError {
    fn from(_err: askama::Error) -> Self {
        EmailConfirmationError::CouldNotSendEmail
    }
}
