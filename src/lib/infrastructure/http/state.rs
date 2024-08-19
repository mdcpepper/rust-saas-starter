//! Application state module

use std::sync::Arc;

use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::auth::services::user::UserService;

/// Application configuration
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// The base URL of the application
    pub base_url: String,

    /// Whether to require email confirmation
    pub require_email_confirmation: bool,
}

/// Global application state
#[derive(Clone)]
pub struct AppState<U: UserService> {
    /// The time the server started
    pub start_time: DateTime<Utc>,

    /// The application configuration
    pub config: AppConfig,

    /// User service
    pub users: Arc<U>,
}

/// Implementation of the application state
impl<U> AppState<U>
where
    U: UserService,
{
    /// Create a new application state
    pub fn new(config: AppConfig, users: U) -> Self {
        Self {
            config,
            start_time: Utc::now(),
            users: Arc::new(users),
        }
    }
}

impl<U> fmt::Debug for AppState<U>
where
    U: UserService,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("start_time", &self.start_time)
            .field("config", &self.config)
            .field("users", &"UserService")
            .finish()
    }
}

#[cfg(test)]
use crate::domain::auth::services::user::MockUserService;

#[cfg(test)]
pub fn test_state(users: Option<MockUserService>) -> AppState<MockUserService> {
    let users = users
        .map(Arc::new)
        .unwrap_or_else(|| Arc::new(MockUserService::new()));

    let config = AppConfig {
        base_url: "https://example.com".to_string(),
        require_email_confirmation: false,
    };

    AppState {
        start_time: Utc::now(),
        config,
        users,
    }
}
