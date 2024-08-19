//! Application state module

use std::sync::Arc;

use std::fmt;

use chrono::{DateTime, Utc};

use crate::domain::auth::services::user::UserServiceImpl;

/// Global application state
#[derive(Clone)]
pub struct AppState<US: UserServiceImpl> {
    /// The time the server started
    pub start_time: DateTime<Utc>,

    /// User service
    pub users: Arc<US>,
}

impl<US> AppState<US>
where
    US: UserServiceImpl,
{
    /// Create a new application state
    pub fn new(users: US) -> Self {
        Self {
            start_time: Utc::now(),
            users: Arc::new(users),
        }
    }
}

impl<US> fmt::Debug for AppState<US>
where
    US: UserServiceImpl,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("start_time", &self.start_time)
            .field("users", &"UserService")
            .finish()
    }
}
