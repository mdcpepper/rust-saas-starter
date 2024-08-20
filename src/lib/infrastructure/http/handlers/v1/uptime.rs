//! Uptime handler

use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    domain::auth::users::UserService,
    domain::communication::email_addresses::EmailAddressService,
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
        (status = StatusCode::OK, description = "Uptime response", body = UptimeResponse),
        (status = StatusCode::TOO_MANY_REQUESTS, description = "Too many requests"),
    )
)]
pub async fn handler<U: UserService, E: EmailAddressService>(
    State(state): State<AppState<U, E>>,
) -> Result<Json<UptimeResponse>, ApiError> {
    let uptime = Utc::now().timestamp() - state.start_time.timestamp();

    Ok(Json(UptimeResponse { uptime }))
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use chrono::Utc;
    use testresult::TestResult;

    use crate::infrastructure::http::{
        handlers::v1::uptime::UptimeResponse, servers::https::router, state::tests::test_state,
    };

    #[tokio::test]
    async fn test_uptime_handler() -> TestResult {
        let state = test_state(None, None);
        let start_time = state.start_time.clone();

        let response = TestServer::new(router(state))?.get("/api/v1/uptime").await;

        let json = response.json::<UptimeResponse>();

        assert_eq!(
            json.uptime,
            Utc::now().timestamp() - start_time.timestamp(),
            "App uptime should be equal to the start time"
        );

        response.assert_status_ok();

        Ok(())
    }
}
