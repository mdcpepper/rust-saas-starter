//! Error types for the email module

use thiserror::Error;

/// Email errors
#[derive(Debug, Error)]
pub enum EmailError {
    /// An error occurred while sending the email
    #[error("An error occurred while sending the email")]
    SendError,

    /// Unknown error
    #[error(transparent)]
    UnknownError(anyhow::Error),
}
