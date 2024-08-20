//! Confirm email template

use anyhow::Result;
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
            link: format!("{base_url}/api/v1/users/{user_id}/email/confirmation?token={token}"),
        }
    }

    /// Renders the plain text version of the email
    pub fn render_plain(&self) -> Result<String> {
        Ok(format!(
            "Visit the following URL to confirm your email address: {link}",
            link = self.link
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_email_address_confirmation_url() {
        let base_url = "https://example.com";
        let user_id = Uuid::now_v7();
        let token = "f9l4Cu5Mpwxu48ITlEfh3QNCgRrda_p23dtSx-ETfkY=";

        let template = ConfirmEmailAddressTemplate::new(base_url, &user_id, token);

        assert_eq!(
            template.link,
            format!("https://example.com/api/v1/users/{user_id}/email/confirmation?token=f9l4Cu5Mpwxu48ITlEfh3QNCgRrda_p23dtSx-ETfkY=")
        );
    }
}
