//! SMTP email service implementation

use anyhow::Result;
use axum::async_trait;
use clap::Parser;
use lettre::{
    address::AddressError,
    error::Error,
    message::MultiPart,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    SmtpTransport, Transport,
};

use crate::domain::communication::mailer::{Mailer, MailerError, Message};

/// SMTP configuration
#[derive(Clone, Default, Debug, Parser)]
pub struct SMTPConfig {
    /// The SMTP host
    #[clap(long, env = "SMTP_HOST")]
    pub host: String,

    /// The SMTP port
    #[clap(long, env = "SMTP_PORT")]
    pub port: u16,

    /// The SMTP username
    #[clap(long, env = "SMTP_USER")]
    pub username: String,

    /// The SMTP password
    #[clap(long, env = "SMTP_PASSWORD")]
    pub password: String,

    /// The sender email address
    #[clap(long, env = "SMTP_SENDER")]
    pub sender: String,

    /// Verify the TLS certificate
    #[clap(long, env = "SMTP_VERIFY_CERTS", default_value = "true")]
    pub verify_certs: bool,

    /// Enable STARTTLS (TLS upgrade on connection)
    #[clap(long, env = "SMTP_STARTTLS", default_value = "true")]
    pub starttls: bool,
}

/// SMTP mailer
#[derive(Debug, Default, Clone)]
pub struct SMTPMailer {
    config: SMTPConfig,
}

impl SMTPMailer {
    /// Create a new SMTP mailer
    pub fn new(config: SMTPConfig) -> Self {
        Self { config }
    }

    /// Create a new SMTP mailer from environment variables
    pub fn mailer(&self) -> Result<SmtpTransport> {
        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());

        let relay = if self.config.starttls {
            SmtpTransport::starttls_relay(&self.config.host)?
        } else {
            SmtpTransport::relay(&self.config.host)?
        };

        Ok(relay
            .credentials(creds)
            .port(self.config.port)
            .tls(Tls::Opportunistic(
                TlsParameters::builder(self.config.host.to_string())
                    .dangerous_accept_invalid_certs(!self.config.verify_certs)
                    .build()?,
            ))
            .build())
    }
}

#[async_trait]
impl Mailer for SMTPMailer {
    async fn send_email(&self, message: Message) -> Result<(), MailerError> {
        let from = if let Some(from) = message.from {
            from.to_string()
        } else {
            self.config.sender.clone()
        };

        let email = lettre::Message::builder()
            .from(from.parse()?)
            .to(message.to.to_string().parse()?)
            .subject(message.subject)
            .multipart(MultiPart::alternative_plain_html(
                String::from(message.plain_body),
                String::from(message.html_body),
            ))?;

        match self.mailer()?.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(MailerError::UnknownError(e.into())),
        }
    }
}

impl From<AddressError> for MailerError {
    fn from(_err: AddressError) -> Self {
        MailerError::InvalidEmail
    }
}

impl From<Error> for MailerError {
    fn from(err: Error) -> Self {
        MailerError::UnknownError(err.into())
    }
}
