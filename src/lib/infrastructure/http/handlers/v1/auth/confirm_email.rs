use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::auth::{errors::EmailConfirmationError, services::user::UserService},
    infrastructure::http::{
        state::AppState,
        templates::{
            auth::email_confirmed::EmailConfirmedTemplate,
            errors::{
                internal_server_error::InternalServerErrorTemplate,
                not_found::NotFoundErrorTemplate,
                unprocessable_entity::UnprocessableEntityErrorTemplate,
            },
        },
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ConfirmEmailParams {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmailConfirmedResponse {
    success: bool,
}

pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ConfirmEmailParams>,
) -> (StatusCode, impl IntoResponse) {
    match state.users.confirm_email(&user_id, &query.token).await {
        Ok(_) => (StatusCode::OK, EmailConfirmedTemplate.into_response()),
        Err(err) => match err {
            EmailConfirmationError::UserNotFound(_) => {
                (StatusCode::NOT_FOUND, NotFoundErrorTemplate.into_response())
            }
            EmailConfirmationError::ConfirmationTokenExpired
            | EmailConfirmationError::ConfirmationTokenMismatch => (
                StatusCode::UNPROCESSABLE_ENTITY,
                UnprocessableEntityErrorTemplate.into_response(),
            ),
            EmailConfirmationError::EmailAlreadyConfirmed => (
                StatusCode::CONFLICT,
                UnprocessableEntityErrorTemplate.into_response(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                InternalServerErrorTemplate.into_response(),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::auth::{errors::EmailConfirmationError, services::user::MockUserService},
        infrastructure::http::{servers::https::router, state::test_state},
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

        response.assert_text_contains("Your email address has been confirmed.");
        response.assert_status(StatusCode::OK);

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

        response.assert_status(StatusCode::NOT_FOUND);
        response.assert_text_contains("Not found.");

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

        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);

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

        response.assert_status(StatusCode::CONFLICT);

        Ok(())
    }
}
