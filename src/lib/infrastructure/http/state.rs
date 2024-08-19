//! Application state module

use std::sync::Arc;

use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::auth::services::user::UserService;

/// Global application state
#[derive(Clone)]
pub struct AppState<U: UserService> {
    /// The time the server started
    pub start_time: DateTime<Utc>,

    /// User service
    pub users: Arc<U>,
}

/// Implementation of the application state
impl<U> AppState<U>
where
    U: UserService,
{
    /// Create a new application state
    pub fn new(users: U) -> Self {
        Self {
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
            .field("users", &"UserService")
            .finish()
    }
}
