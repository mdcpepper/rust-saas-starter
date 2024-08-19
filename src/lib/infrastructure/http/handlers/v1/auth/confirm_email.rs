use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::auth::services::user::UserService,
    infrastructure::http::{errors::ApiError, state::AppState},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmEmailParams {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmailConfirmedResponse {
    success: bool,
}

/// Confirm the email of a user
#[utoipa::path(
    get,
    operation_id = "confirm_email",
    tag = "Auth",
    path = "/api/v1/users/{id}/email/confirmation",
    params(
        ("id" = Uuid, Path, description = "The UUID of the user", example = "550e8400-e29b-41d4-a716-446655440000"),
        ("token" = String, Query, description = "The email confirmation token", example = "f9l4Cu5Mpwxu48ITlEfh3QNCgRrda_p23dtSx-ETfkY=")
    ),
    responses(
        (status = StatusCode::OK, description = "Email confirmed", body = EmailConfirmedResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse, example = json!({ "error": "User with id \"550e8400-e29b-41d4-a716-446655440000\" not found" })),
        (status = StatusCode::BAD_REQUEST, description = "Invalid token", body = ErrorResponse, example = json!({ "error": "Invalid email confirmation token" })),
        (status = StatusCode::CONFLICT, description = "Email already confirmed", body = ErrorResponse, example = json!({ "error": "Email already confirmed" })),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = ErrorResponse, example = json!({ "error": "Failed to confirm email: <error>" })),
    )
)]
pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ConfirmEmailParams>,
) -> Result<Json<EmailConfirmedResponse>, ApiError> {
    state.users.confirm_email(&user_id, &query.token).await?;

    Ok(Json(EmailConfirmedResponse { success: true }))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::auth::{errors::EmailConfirmationError, services::user::MockUserService},
        infrastructure::http::{
            errors::ErrorResponse, handlers::v1::auth::confirm_email::EmailConfirmedResponse,
            servers::https::router, state::test_state,
        },
    };

    #[tokio::test]
    async fn test_confirm_email_success() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();

        users
            .expect_confirm_email()
            .times(1)
            .withf(move |id, token| *id == user_id.clone() && token == "test-token")
            .returning(move |_, _| Ok(()));

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .get(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .add_raw_query_param("token=test-token")
            .await;

        let json = response.json::<EmailConfirmedResponse>();

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(json.success, true);

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_user_not_found() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();

        users
            .expect_confirm_email()
            .times(1)
            .withf(move |id, token| *id == user_id.clone() && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::UserNotFound(user_id.clone())));

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .get(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .add_query_param("token", "test-token")
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            json.error,
            format!("User with id \"{}\" not found", user_id)
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_invalid_token() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();

        users
            .expect_confirm_email()
            .times(1)
            .withf(move |id, token| *id == user_id.clone() && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::ConfirmationTokenMismatch));

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .get(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .add_query_param("token", "test-token")
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json.error, "Confirmation token does not match");

        Ok(())
    }

    #[tokio::test]
    async fn test_confirm_email_already_confirmed() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();

        users
            .expect_confirm_email()
            .times(1)
            .withf(move |id, token| *id == user_id.clone() && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::EmailAlreadyConfirmed));

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .get(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .add_query_param("token", "test-token")
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::CONFLICT);
        assert_eq!(json.error, "Email is already confirmed");

        Ok(())
    }
}
