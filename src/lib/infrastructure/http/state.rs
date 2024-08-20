//! Application state module

use std::sync::Arc;

use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::{
    auth::users::UserService, communication::email_addresses::EmailAddressService,
};

/// Application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// The base URL of the application
    pub base_url: String,
}

/// Global application state
#[derive(Clone)]
pub struct AppState<U: UserService, E: EmailAddressService> {
    /// The time the server started
    pub start_time: DateTime<Utc>,

    /// The application configuration
    pub config: AppConfig,

    /// User service
    pub users: Arc<U>,

    /// Email address service
    pub email_addresses: Arc<E>,
}

/// Implementation of the application state
impl<U, E> AppState<U, E>
where
    U: UserService,
    E: EmailAddressService,
{
    /// Create a new application state
    pub fn new(config: AppConfig, users: U, email_addresses: E) -> Self {
        Self {
            config,
            start_time: Utc::now(),
            users: Arc::new(users),
            email_addresses: Arc::new(email_addresses),
        }
    }
}

impl<U, E> fmt::Debug for AppState<U, E>
where
    U: UserService,
    E: EmailAddressService,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("start_time", &self.start_time)
            .field("config", &self.config)
            .field("users", &"UserService")
            .field("email_addresses", &"EmailAddressService")
            .finish()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::domain::{
        auth::users::tests::MockUserService,
        communication::email_addresses::tests::MockEmailAddressService,
    };

    use super::*;

    pub fn test_state(
        users: Option<MockUserService>,
        email_addresses: Option<MockEmailAddressService>,
    ) -> AppState<MockUserService, MockEmailAddressService> {
        let users = users
            .map(Arc::new)
            .unwrap_or_else(|| Arc::new(MockUserService::new()));

        let email_addresses = email_addresses
            .map(Arc::new)
            .unwrap_or_else(|| Arc::new(MockEmailAddressService::new()));

        let config = AppConfig {
            base_url: "https://example.com".to_string(),
        };

        AppState {
            start_time: Utc::now(),
            config,
            users,
            email_addresses,
        }
    }
}
