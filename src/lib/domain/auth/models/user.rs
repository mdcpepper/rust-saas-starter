//! User model

use chrono::{DateTime, Utc};
use password_auth::generate_hash;
use uuid::Uuid;

use crate::domain::auth::value_objects::{email_address::EmailAddress, password::Password};

/// User model
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    /// User UUID
    pub id: Uuid,

    /// User email address
    pub email: EmailAddress,

    /// User created at date in UTC
    pub created_at: DateTime<Utc>,

    /// User last updated at date in UTC
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Get the user's id
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the user's email address
    pub fn email(&self) -> &EmailAddress {
        &self.email
    }

    /// Get the user's created at date
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Get the user's updated at date
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
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
}

impl NewUser {
    /// Create a new user request
    pub fn new(id: Uuid, email: EmailAddress, password: Password) -> Self {
        let password_hash = generate_hash(password.as_bytes());

        Self {
            id,
            email,
            password_hash,
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

    use crate::domain::auth::value_objects::{email_address::EmailAddress, password::Password};

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
