//! SMTP email service implementation

use anyhow::Result;
use axum::async_trait;
use clap::Parser;
use lettre::{
    message::MultiPart,
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
    Message, SmtpTransport, Transport,
};

use crate::domain::comms::{
    errors::EmailError, mailer::Mailer, value_objects::email_address::EmailAddress,
};

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
    #[clap(long, env = "SMTP_VERIFY_TLS", default_value = "true")]
    pub verify_tls: bool,

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
                    .dangerous_accept_invalid_certs(!self.config.verify_tls)
                    .build()?,
            ))
            .build())
    }
}

#[async_trait]
impl Mailer for SMTPMailer {
    async fn send_email(
        &self,
        to: &EmailAddress,
        subject: &str,
        html: &str,
        plain: &str,
    ) -> Result<(), EmailError> {
        let email = Message::builder()
            .from(self.config.sender.parse()?)
            .to(to.to_string().parse()?)
            .subject(subject.to_string())
            .multipart(MultiPart::alternative_plain_html(
                String::from(plain),
                String::from(html),
            ))?;

        match self.mailer()?.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(EmailError::UnknownError(e.into())),
        }
    }
}
