use css_inline::InlineError;
use thiserror::Error;
use tracing::debug;

use crate::domain::{
    auth::users::errors::{GetUserByIdError, UpdateUserError},
    communication::mailer::MailerError,
};

/// Errors that can occur when sending or verifying an email confirmation
#[derive(Debug, Error)]
pub enum EmailConfirmationError {
    /// User not found
    #[error("user not found")]
    UserNotFound,

    /// Email already in use
    #[error("email is already in use")]
    EmailAddressInUse,

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

impl From<GetUserByIdError> for EmailConfirmationError {
    fn from(err: GetUserByIdError) -> Self {
        debug!("GetUserByIdError -> EmailConfirmationError");

        match err {
            GetUserByIdError::UserNotFound => EmailConfirmationError::UserNotFound,
            GetUserByIdError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
        }
    }
}

impl From<UpdateUserError> for EmailConfirmationError {
    fn from(err: UpdateUserError) -> Self {
        debug!("UpdateUserError -> EmailConfirmationError");

        match err {
            UpdateUserError::UserNotFound => EmailConfirmationError::UserNotFound,
            UpdateUserError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
            UpdateUserError::EmailAddressInUse => EmailConfirmationError::EmailAddressInUse,
        }
    }
}

impl From<MailerError> for EmailConfirmationError {
    fn from(err: MailerError) -> Self {
        debug!("MailerError -> EmailConfirmationError");

        match err {
            MailerError::SendError | MailerError::InvalidEmail => {
                EmailConfirmationError::CouldNotSendEmail
            }
            MailerError::UnknownError(e) => EmailConfirmationError::UnknownError(e),
        }
    }
}

impl From<InlineError> for EmailConfirmationError {
    fn from(_err: InlineError) -> Self {
        debug!("InlineError -> EmailConfirmationError");

        EmailConfirmationError::CouldNotSendEmail
    }
}

impl From<askama::Error> for EmailConfirmationError {
    fn from(_err: askama::Error) -> Self {
        debug!("askama::Error -> EmailConfirmationError");

        EmailConfirmationError::CouldNotSendEmail
    }
}
