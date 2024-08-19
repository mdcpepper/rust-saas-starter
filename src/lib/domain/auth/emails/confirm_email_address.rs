//! Confirm email template

use askama::Template;
use uuid::Uuid;

/// Confirm email address template
#[derive(Debug, Template)]
#[template(path = "emails/auth/confirm_email_address.html")]
pub struct ConfirmEmailAddressTemplate {
    /// Link to confirm email address
    pub link: String,
}

impl ConfirmEmailAddressTemplate {
    /// Creates a new `ConfirmEmailAddressTemplate`
    pub fn new(base_url: &str, user_id: &Uuid, token: &str) -> Self {
        Self {
            link: format!(
                "{}/users/{}/email/confirmation/confirm?token={}",
                base_url, user_id, token
            ),
        }
    }
}
