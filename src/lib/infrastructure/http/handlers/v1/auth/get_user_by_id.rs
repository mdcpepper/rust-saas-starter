//! Get User by ID

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        auth::users::{User, UserService},
        communication::email_addresses::EmailAddressService,
    },
    infrastructure::http::{errors::ApiError, state::AppState},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetUserByIdResponse {
    id: String,
    email: String,
    email_confirmed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<User> for GetUserByIdResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email.to_string(),
            email_confirmed_at: user.email_confirmed_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Get a user by their ID
#[utoipa::path(
    get,
    operation_id = "get_user_by_id",
    tag = "Auth",
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "The UUID of the user", example = "550e8400-e29b-41d4-a716-446655440000"),
    ),
    responses(
        (status = StatusCode::OK, description = "User found", body = GetUserByIdResponse),
        (status = StatusCode::NOT_FOUND, description = "User not found", body = ErrorResponse),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "Internal Server Error", body = ErrorResponse),
        (status = StatusCode::TOO_MANY_REQUESTS, description = "Too many requests"),
    )
)]
pub async fn handler<U: UserService, E: EmailAddressService>(
    State(state): State<AppState<U, E>>,
    Path(id): Path<Uuid>,
) -> Result<Json<GetUserByIdResponse>, ApiError> {
    let user = state.users.get_user_by_id(&id).await?.into();

    Ok(Json(user))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use chrono::Utc;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::{
            auth::users::{errors::GetUserByIdError, tests::MockUserService, User},
            communication::email_addresses::EmailAddress,
        },
        infrastructure::http::{
            errors::ErrorResponse, handlers::v1::auth::get_user_by_id::GetUserByIdResponse,
            servers::https::router, state::tests::test_state,
        },
    };

    #[tokio::test]
    async fn test_get_user_by_id_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            email_confirmed_at: None,
            email_confirmation_token: None,
            email_confirmation_sent_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut users = MockUserService::new();

        users
            .expect_get_user_by_id()
            .withf(move |id| *id == user.id)
            .returning(move |_| Ok(user.clone()));

        let state = test_state(Some(users), None);

        let response = TestServer::new(router(state))?
            .get(&format!("/api/v1/users/{}", user_id.clone()))
            .await;

        let json = response.json::<GetUserByIdResponse>();

        assert_eq!(response.status_code(), StatusCode::OK);

        assert_eq!(user_id.to_string(), json.id.to_string());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() -> TestResult {
        let user_id = Uuid::now_v7();
        let expected_user_id = user_id.clone();
        let mut users = MockUserService::new();

        users
            .expect_get_user_by_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| Err(GetUserByIdError::UserNotFound(user_id.clone())));

        let state = test_state(Some(users), None);

        let response = TestServer::new(router(state))?
            .get(&format!("/api/v1/users/{user_id}"))
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            json.error,
            format!("User with id \"{expected_user_id}\" not found")
        );

        Ok(())
    }
}
