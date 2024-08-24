//! Send change email confirmation email handler

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        auth::users::UserService,
        communication::email_addresses::{
            EmailAddress, EmailAddressService, EmailConfirmationType,
        },
    },
    infrastructure::http::{errors::ApiError, state::AppState},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangeEmailRequest {
    #[schema(value_type = String, example = "email@example.com")]
    email: EmailAddress,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChangeEmailResponse {
    expires_at: DateTime<Utc>,
}

/// Send a change email confirmation email
#[utoipa::path(
    post,
    operation_id = "send_change_email_confirmation",
    tag = "Auth",
    path = "/api/v1/users/{id}/email/change",
    request_body = ChangeEmailRequest,
    params(
        ("id" = Uuid, Path, description = "The UUID of the user", example = "550e8400-e29b-41d4-a716-446655440000"),
    ),
    responses(
        (status = StatusCode::ACCEPTED, description = "Email change confirmation sent", body = ChangeEmailResponse),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "Unprocessable entity", body = ErrorResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse, example = json!({ "error": "User with id \"550e8400-e29b-41d4-a716-446655440000\" not found" })),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = ErrorResponse, example = json!({ "error": "Failed to send email confirmation: <error>" })),
        (status = StatusCode::TOO_MANY_REQUESTS, description = "Too many requests", body = TooManyRequestsResponse),
    )
)]
pub async fn handler<U: UserService, E: EmailAddressService>(
    State(state): State<AppState<U, E>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<ChangeEmailRequest>,
) -> Result<(StatusCode, Json<ChangeEmailResponse>), ApiError> {
    let user = state.users.get_user_by_id(&user_id).await?;

    let expires_at = state
        .email_addresses
        .send_email_confirmation(
            &user,
            EmailConfirmationType::NewEmail(request.email),
            &state.config.base_url,
        )
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ChangeEmailResponse { expires_at }),
    ))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use chrono::{Duration, Utc};
    use serde_json::json;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::{
            auth::users::{tests::MockUserService, User},
            communication::email_addresses::{
                tests::MockEmailAddressService, EmailAddress, EmailConfirmationType,
            },
        },
        infrastructure::http::{servers::https::router, state::tests::test_state},
    };

    use super::ChangeEmailResponse;

    #[tokio::test]
    async fn test_send_change_email_confirmation() -> TestResult {
        let user_id = Uuid::now_v7();
        let yesterday = Utc::now() - Duration::days(1);
        let changed_email = EmailAddress::new_unchecked("new_email@example.com");
        let expected_email = changed_email.clone();
        let expected_confirmation_type = EmailConfirmationType::NewEmail(changed_email.clone());

        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            new_email: None,
            email_confirmed_at: None,
            email_confirmation_token: Some("token".to_string()),
            email_confirmation_sent_at: Some(yesterday.clone()),
            created_at: yesterday.clone(),
            updated_at: yesterday.clone(),
        };

        let expected_expiry = Utc::now() + Duration::days(1);

        let mut users = MockUserService::new();
        let mut email_addresses = MockEmailAddressService::new();

        users
            .expect_get_user_by_id()
            .withf(move |id| *id == user.id)
            .returning(move |_| Ok(user.clone()));

        email_addresses
            .expect_send_email_confirmation()
            .times(1)
            .withf(move |user, confirmation_type, base_url| {
                *user == user.clone()
                    && *confirmation_type == expected_confirmation_type
                    && base_url == "https://example.com"
            })
            .returning(move |_, _, _| Ok(expected_expiry.clone()));

        let state = test_state(Some(users), Some(email_addresses));

        let response = TestServer::new(router(state))?
            .post(&format!("/api/v1/users/{}/email/change", user_id.clone()))
            .json(&json!({ "email": expected_email }))
            .await;

        let json = response.json::<ChangeEmailResponse>();

        assert_eq!(response.status_code(), StatusCode::ACCEPTED);
        assert_eq!(json.expires_at, expected_expiry);

        Ok(())
    }
}
