//! User model

use chrono::{DateTime, Utc};
use password_auth::generate_hash;
use uuid::Uuid;

use crate::domain::{
    auth::value_objects::password::Password, comms::value_objects::email_address::EmailAddress,
};

/// User model
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    /// User UUID
    pub id: Uuid,

    /// User email address
    pub email: EmailAddress,

    /// User email confirmed at date in UTC
    pub email_confirmed_at: Option<DateTime<Utc>>,

    /// User email confirmation token
    pub email_confirmation_token: Option<String>,

    /// User email confirmation sent at date in UTC
    pub email_confirmation_sent_at: Option<DateTime<Utc>>,

    /// User created at date in UTC
    pub created_at: DateTime<Utc>,

    /// User last updated at date in UTC
    pub updated_at: DateTime<Utc>,
}

/// Create user request
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUser {
    /// New user's ID
    id: Uuid,

    /// New user's email address
    email: EmailAddress,

    /// New user's password
    password_hash: String,

    /// Whether email confirmation is required
    email_confirmation_required: bool,
}

impl NewUser {
    /// Create a new user request
    pub fn new(id: Uuid, email: EmailAddress, password: Password) -> Self {
        let password_hash = generate_hash(password.as_bytes());

        Self {
            id,
            email,
            password_hash,
            email_confirmation_required: true,
        }
    }

    /// Get the new user's ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the new user's email address
    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    /// Get the new user's password hash
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }
}

#[cfg(test)]
mod tests {
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::domain::{
        auth::value_objects::password::Password, comms::value_objects::email_address::EmailAddress,
    };

    use super::NewUser;

    #[test]
    fn create_user_request_hashes_password() -> TestResult {
        let create_user = NewUser::new(
            Uuid::now_v7(),
            EmailAddress::new("email@example.com")?,
            Password::new("correcthorsebatterystaple")?,
        );

        assert_ne!(create_user.password_hash(), "correcthorsebatterystaple");

        Ok(())
    }
}
