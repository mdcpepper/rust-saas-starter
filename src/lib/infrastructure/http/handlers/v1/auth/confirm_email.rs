use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{ErrorResponse, IntoResponse},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{auth::users::UserService, communication::email_addresses::EmailAddressService},
    infrastructure::http::{
        state::AppState, templates::auth::email_confirmed::EmailConfirmedTemplate,
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

/// Confirm a user's email address
pub async fn handler<U: UserService, E: EmailAddressService>(
    State(state): State<AppState<U, E>>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ConfirmEmailParams>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let user = state.users.get_user_by_id(&user_id).await?;

    state
        .email_addresses
        .confirm_email(&user, &query.token)
        .await?;

    Ok((StatusCode::OK, EmailConfirmedTemplate))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::{
            auth::users::{tests::MockUserService, User},
            communication::email_addresses::{
                tests::MockEmailAddressService, EmailConfirmationError,
            },
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
            .returning(move |_, _| Err(EmailConfirmationError::UserNotFound));

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
