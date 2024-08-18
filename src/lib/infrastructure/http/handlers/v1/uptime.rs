//! Uptime handler

use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    domain::auth::services::user::UserManagement,
    infrastructure::http::{errors::ApiError, state::AppState},
};

/// The uptime response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UptimeResponse {
    /// The uptime of the application in seconds
    #[schema(example = 123)]
    pub uptime: i64,
}

/// Get the uptime of the application
#[utoipa::path(
    get,
    operation_id = "uptime",
    tag = "System",
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

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use chrono::Utc;
    use testresult::TestResult;

    use crate::{
        domain::auth::repositories::user::MockUserRepository,
        infrastructure::http::{
            handlers::v1::uptime::UptimeResponse,
            servers::https::router,
            state::{get_test_state, MockAppState},
        },
    };

    #[tokio::test]
    async fn test_uptime_handler() -> TestResult {
        let state: MockAppState = get_test_state(MockUserRepository::new());

        let response = TestServer::new(router(state.clone()))?
            .get("/api/v1/uptime")
            .await;

        let json = response.json::<UptimeResponse>();

        assert_eq!(
            json.uptime,
            Utc::now().timestamp() - state.start_time.timestamp(),
            "App uptime should be equal to the start time"
        );

        response.assert_status_ok();

        Ok(())
    }
}
