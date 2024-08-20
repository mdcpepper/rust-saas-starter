use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        auth::users::{
            errors::{EmailConfirmationError, GetUserByIdError},
            UserService,
        },
        communication::email_addresses::EmailAddressService,
    },
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

pub async fn handler<U: UserService, E: EmailAddressService>(
    State(state): State<AppState<U, E>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ConfirmEmailParams>,
) -> (StatusCode, impl IntoResponse) {
    let user = state.users.get_user_by_id(&user_id).await;

    match user {
        Ok(user) => match state
            .email_addresses
            .confirm_email(&user, &query.token)
            .await
        {
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
        },
        Err(err) => match err {
            GetUserByIdError::UserNotFound(_) => {
                (StatusCode::NOT_FOUND, NotFoundErrorTemplate.into_response())
            }
            GetUserByIdError::UnknownError(_) => (
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
        domain::{
            auth::users::{errors::EmailConfirmationError, tests::MockUserService, User},
            communication::email_addresses::tests::MockEmailAddressService,
        },
        infrastructure::http::{servers::https::router, state::tests::test_state},
    };

    #[tokio::test]
    async fn test_confirm_email_success() -> TestResult {
        let user_id = Uuid::now_v7();

        let mut users = MockUserService::new();
        let mut email_addresses = MockEmailAddressService::new();

        let user = User::default();
        let expected_user = user.clone();

        users
            .expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(user.clone()));

        email_addresses
            .expect_confirm_email()
            .times(1)
            .withf(move |user, token| *user == expected_user && token == "test-token")
            .returning(move |_, _| Ok(()));

        let state = test_state(Some(users), Some(email_addresses));

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
        let mut email_addresses = MockEmailAddressService::new();

        let user = User::default();
        let expected_user = user.clone();

        users
            .expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(user.clone()));

        email_addresses
            .expect_confirm_email()
            .times(1)
            .withf(move |user, token| *user == expected_user && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::UserNotFound(user_id.clone())));

        let state = test_state(Some(users), Some(email_addresses));

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
        let mut email_addresses = MockEmailAddressService::new();

        let user = User::default();
        let expected_user = user.clone();

        users
            .expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(user.clone()));

        email_addresses
            .expect_confirm_email()
            .times(1)
            .withf(move |user, token| *user == expected_user && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::ConfirmationTokenMismatch));

        let state = test_state(Some(users), Some(email_addresses));

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
        let mut email_addresses = MockEmailAddressService::new();

        let user = User::default();
        let expected_user = user.clone();

        users
            .expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(user.clone()));

        email_addresses
            .expect_confirm_email()
            .times(1)
            .withf(move |user, token| *user == expected_user && token == "test-token")
            .returning(move |_, _| Err(EmailConfirmationError::EmailAlreadyConfirmed));

        let state = test_state(Some(users), Some(email_addresses));

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
