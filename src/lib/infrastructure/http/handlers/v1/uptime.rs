//! Uptime handler

use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    domain::auth::services::user::UserManagement,
    infrastructure::http::{errors::ApiError, state::AppState},
};

/// The uptime response
#[derive(Debug, Serialize, ToSchema)]
pub struct UptimeResponse {
    /// The uptime of the application in seconds
    #[schema(example = 123)]
    pub uptime: i64,
}

/// Get the uptime of the application
#[utoipa::path(
    get,
    operation_id = "uptime",
    path = "/api/v1/uptime",
    responses(
        (status = 200, description = "Uptime response", body = UptimeResponse),
    )
)]
pub async fn handler<US: UserManagement>(
    State(state): State<AppState<US>>,
) -> Result<Json<UptimeResponse>, ApiError> {
    let uptime = Utc::now().timestamp() - state.start_time.timestamp();

    Ok(Json(UptimeResponse { uptime }))
}
