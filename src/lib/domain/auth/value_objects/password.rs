//! Password

use std::fmt;

use thiserror::Error;
use zxcvbn::{zxcvbn, Score};

/// Password error
#[derive(Debug, Error)]
pub enum PasswordError {
    /// Password is too short
    #[error("Your password is too short. It must be at least 8 characters long.")]
    TooShort,

    /// Password is too long
    #[error("Your password is too long. It must be at most 100 characters long.")]
    TooLong,

    /// Password is too weak
    #[error("Your password is too weak.")]
    TooWeak(Vec<String>),
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

        if raw.len() > 100 {
            return Err(PasswordError::TooLong);
        }

        let entropy = zxcvbn(raw, &[]);
        if entropy.score() < Score::Three {
            let suggestions = if let Some(feedback) = entropy.feedback() {
                feedback
                    .suggestions()
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            } else {
                vec!["Please choose a stronger password.".to_string()]
            };

            return Err(PasswordError::TooWeak(suggestions));
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
    fn test_password_display_obfuscates() -> TestResult {
        let password = Password::new("correcthorsebatterystaple")?;
        assert_eq!(format!("{}", password), "********");

        Ok(())
    }

    #[test]
    fn test_password_debug_obfuscates() -> TestResult {
        let password = Password::new("correcthorsebatterystaple")?;
        assert_eq!(format!("{:?}", password), "********");

        Ok(())
    }

    #[test]
    fn test_get_password_as_bytes() -> TestResult {
        let password = Password::new("correcthorsebatterystaple")?;
        assert_eq!(password.as_bytes(), b"correcthorsebatterystaple");

        Ok(())
    }

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
    fn test_new_password_too_long() {
        let result = Password::new(&"a".repeat(101));
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::TooLong)))
    }

    #[test]
    fn test_new_password_too_weak() {
        let result = Password::new("weakpassword");
        assert!(result.is_err());
        assert!(matches!(result, Err(PasswordError::TooWeak(_))));
    }
}
