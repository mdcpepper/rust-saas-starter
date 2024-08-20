//! Send email confirmation email handler

use axum::{
    extract::{Path, State},
    http::StatusCode,
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
pub struct SendEmailConfirmationResponse {
    success: bool,
}

/// Send an email confirmation email
#[utoipa::path(
    post,
    operation_id = "send_email_confirmation",
    tag = "Auth",
    path = "/api/v1/users/{id}/email/confirmation",
    params(
        ("id" = Uuid, Path, description = "The UUID of the user", example = "550e8400-e29b-41d4-a716-446655440000"),
    ),
    responses(
        (status = StatusCode::ACCEPTED, description = "Email confirmation sent", body = SendEmailConfirmationResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse, example = json!({ "error": "User with id \"550e8400-e29b-41d4-a716-446655440000\" not found" })),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = ErrorResponse, example = json!({ "error": "Failed to send email confirmation: <error>" })),
    )
)]
pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    Path(user_id): Path<Uuid>,
) -> Result<(StatusCode, Json<SendEmailConfirmationResponse>), ApiError> {
    state
        .users
        .send_email_confirmation(&user_id, &state.config.base_url)
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(SendEmailConfirmationResponse { success: true }),
    ))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use chrono::{Duration, Utc};
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::{
            auth::{
                errors::EmailConfirmationError, models::user::User, services::user::MockUserService,
            },
            communication::value_objects::email_address::EmailAddress,
        },
        infrastructure::http::{
            errors::ErrorResponse,
            handlers::v1::auth::send_email_confirmation::SendEmailConfirmationResponse,
            servers::https::router, state::test_state,
        },
    };

    #[tokio::test]
    async fn test_send_email_confirmation_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let yesterday = Utc::now() - Duration::days(1);

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: Some("token".to_string()),
            email_confirmation_sent_at: Some(yesterday.clone()),
            created_at: yesterday.clone(),
            updated_at: yesterday.clone(),
        };

        let mut users = MockUserService::new();

        users
            .expect_get_user_by_id()
            .withf(move |id| *id == user.id)
            .returning(move |_| Ok(user.clone()));

        users
            .expect_send_email_confirmation()
            .times(1)
            .withf(move |id, base_url| *id == user_id.clone() && base_url == "https://example.com")
            .returning(move |_, _| Ok(Utc::now() + Duration::days(1)));

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .post(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .await;

        let json = response.json::<SendEmailConfirmationResponse>();

        assert_eq!(response.status_code(), StatusCode::ACCEPTED);
        assert_eq!(json.success, true);

        Ok(())
    }

    #[tokio::test]
    async fn test_send_email_confirmation_user_not_found() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();

        users
            .expect_send_email_confirmation()
            .times(1)
            .withf(move |id, base_url| *id == user_id && base_url == "https://example.com")
            .returning(move |user_id, _| {
                Err(EmailConfirmationError::UserNotFound(user_id.clone()))
            });

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .post(&format!(
                "/api/v1/users/{}/email/confirmation",
                user_id.clone()
            ))
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            json.error,
            format!("User with id \"{}\" not found", user_id)
        );

        Ok(())
    }
}
