//! Mailer module

mod errors;

pub use errors::MailerError;

use async_trait::async_trait;
use mockall::mock;

use crate::domain::communication::email_addresses::EmailAddress;

/// Mailer trait
#[async_trait]
pub trait Mailer: Clone + Send + Sync + 'static {
    /// Send an email
    ///
    /// # Arguments
    /// * `to` - The [`EmailAddress`] to send the email to.
    /// * `subject` - The subject of the email.
    /// * `body` - The body of the email.
    /// * `plain` - The plain text version of the email.
    /// * `html` - The HTML version of the email.
    ///
    /// # Returns
    /// A [`Result`] indicating success or failure.
    async fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        html: &str,
        plain: &str,
    ) -> Result<(), MailerError>;
}

mock! {
    pub Mailer {}

    impl Clone for Mailer {
        fn clone(&self) -> Self;
    }

    #[async_trait]
    impl Mailer for Mailer {
        async fn send_email(&self, to: &EmailAddress, subject: &str, html: &str, plain: &str) -> Result<(), MailerError>;
    }
}

#[cfg(test)]
pub mod tests {
    pub use super::MockMailer;
}
