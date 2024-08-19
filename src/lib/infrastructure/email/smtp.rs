//! SMTP email service implementation

use axum::async_trait;

use crate::domain::comms::{
    errors::EmailError, mailer::Mailer, value_objects::email_address::EmailAddress,
};

/// SMTP mailer
#[derive(Debug, Clone)]
pub struct SMTPMailer;

impl SMTPMailer {
    /// Create a new SMTP mailer
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Mailer for SMTPMailer {
    async fn send_email(
        &self,
        _to: &EmailAddress,
        _subject: &str,
        _body: &str,
    ) -> Result<(), EmailError> {
        todo!()
    }
}
