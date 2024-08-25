//! Email message

use crate::domain::communication::email_addresses::EmailAddress;

/// Email message
#[derive(Debug)]
pub struct Message {
    /// The recipient of the email
    pub to: EmailAddress,

    /// The sender of the email
    pub from: Option<EmailAddress>,

    /// The subject of the email
    pub subject: String,

    /// The HTML body of the email
    pub html_body: String,

    /// The plain text body of the email
    pub plain_body: String,
}
