//! Password

use std::fmt;

use thiserror::Error;
use zxcvbn::{zxcvbn, Score};

/// Password error
#[derive(Debug, Error)]
pub enum PasswordError {
    /// Password is too short
    #[error("password is too short")]
    TooShort,

    /// Password is too weak
    #[error("password is too weak")]
    TooWeak,
}

/// Password
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    /// Create a new password
    pub fn new(raw: &str) -> Result<Self, PasswordError> {
        if raw.len() < 8 {
            return Err(PasswordError::TooShort);
        }

        if zxcvbn(raw, &[]).score() < Score::Three {
            return Err(PasswordError::TooWeak);
        }

        Ok(Self(raw.to_string()))
    }

    /// Get the password as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "********")
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "********")
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;

    use super::*;

    #[test]
    fn test_new_password() -> TestResult {
        let password = Password::new("correcthorsebatterystaple")?;
        assert_eq!(password.to_string(), "********");
        Ok(())
    }

    #[test]
    fn test_new_password_too_short() {
        let result = Password::new("short");
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::TooShort)))
    }

    #[test]
    fn test_new_password_too_weak() {
        let result = Password::new("weakpassword");
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::TooWeak)))
    }
}
