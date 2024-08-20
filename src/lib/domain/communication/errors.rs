//! Error types for the email module

use lettre::{address::AddressError, error::Error};
use thiserror::Error;

/// Email errors
#[derive(Debug, Error)]
pub enum EmailError {
    /// An error occurred while sending the email
    #[error("An error occurred while sending the email")]
    SendError,

    /// Invalid email address
    #[error("Invalid email address")]
    InvalidEmail,

    /// Unknown error
    #[error(transparent)]
    UnknownError(anyhow::Error),
}

impl From<anyhow::Error> for EmailError {
    fn from(err: anyhow::Error) -> Self {
        EmailError::UnknownError(err)
    }
}

impl From<AddressError> for EmailError {
    fn from(_err: AddressError) -> Self {
        EmailError::InvalidEmail
    }
}

impl From<Error> for EmailError {
    fn from(err: Error) -> Self {
        EmailError::UnknownError(err.into())
    }
}
