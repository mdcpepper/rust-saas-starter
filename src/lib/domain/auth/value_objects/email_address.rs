//! Email Address

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[^@\s]*?@[^@\s]*?\.[^@\s]*$").unwrap();
}

use std::fmt;

use thiserror::Error;

use EmailAddressError::*;

/// An error that can occur when creating an email address
#[derive(Debug, Error)]
pub enum EmailAddressError {
    /// The email address is empty
    #[error("email is empty")]
    EmptyEmailAddress,

    /// The email address is invalid
    #[error("email is invalid")]
    InvalidEmailAddress,
}

/// An email address
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Create a new email address
    pub fn new(raw: &str) -> Result<Self, EmailAddressError> {
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(EmptyEmailAddress);
        }

        if !EMAIL_REGEX.is_match(trimmed) {
            return Err(EmailAddressError::InvalidEmailAddress);
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<EmailAddress> for String {
    fn from(email: EmailAddress) -> Self {
        email.0
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_email_address_display() -> TestResult {
        let email = EmailAddress::new("email@example.com")?;

        assert_eq!(format!("{}", email), "email@example.com".to_string());

        Ok(())
    }

    #[test]
    fn test_empty_email_address_is_invalid() {
        let result = EmailAddress::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), EmptyEmailAddress));
    }

    #[test]
    fn test_email_address_without_at_symbol_is_invalid() {
        let result = EmailAddress::new("email");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), InvalidEmailAddress));
    }

    #[test]
    fn test_valid_email_to_string() -> TestResult {
        let email = EmailAddress::new("email@example.com")?;

        assert_eq!(String::from(email), "email@example.com".to_string());

        Ok(())
    }
}
