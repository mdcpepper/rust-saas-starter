//! User model

use thiserror::Error;
use uuid::Uuid;

use crate::domain::auth::value_objects::email_address::EmailAddress;

/// User model
#[derive(Debug)]
pub struct User {
    /// User UUID
    id: Uuid,

    /// User email address
    email: EmailAddress,
}

impl User {
    /// Create a new user
    pub fn new(id: Uuid, email: EmailAddress) -> Self {
        Self { id, email }
    }

    /// Get the user's id
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the user's email address
    pub fn email(&self) -> &EmailAddress {
        &self.email
    }
}

/// Create user request
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateUserRequest {
    /// New user's ID
    id: Uuid,

    /// New user's email address
    email: EmailAddress,
}

impl CreateUserRequest {
    /// Create a new user request
    pub fn new(id: Uuid, email: EmailAddress) -> Self {
        Self { id, email }
    }

    /// Get the new user's ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get the new user's email address
    pub fn email(&self) -> &EmailAddress {
        &self.email
    }
}

/// Errors that can occur when creating a user
#[derive(Debug, Error)]
pub enum CreateUserError {
    /// User with email already exists
    #[error("user with email {email} already exists")]
    DuplicateUser {
        /// Email address
        email: EmailAddress,
    },

    /// Unknown error
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}
