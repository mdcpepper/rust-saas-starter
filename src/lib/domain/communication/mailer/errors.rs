//! Mailer errors

use thiserror::Error;

/// Mailer errors
#[derive(Debug, Error)]
pub enum MailerError {
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

impl From<anyhow::Error> for MailerError {
    fn from(err: anyhow::Error) -> Self {
        MailerError::UnknownError(err)
    }
}
