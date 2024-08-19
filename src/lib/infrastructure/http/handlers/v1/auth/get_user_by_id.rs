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
    domain::auth::{models::user::User, services::user::UserService},
    infrastructure::http::{errors::ApiError, state::AppState},
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetUserByIdResponse {
    id: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<User> for GetUserByIdResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id().to_string(),
            email: user.email().to_string(),
            created_at: *user.created_at(),
            updated_at: *user.updated_at(),
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
    )
)]
pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    Path(id): Path<Uuid>,
) -> Result<Json<GetUserByIdResponse>, ApiError> {
    let user = state.users.get_user_by_id(&id).await?;

    Ok(Json(user.into()))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use chrono::Utc;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::auth::{
            errors::GetUserByIdError, models::user::User, services::user::MockUserService,
            value_objects::email_address::EmailAddress,
        },
        infrastructure::http::{
            errors::ErrorResponse, handlers::v1::auth::get_user_by_id::GetUserByIdResponse,
            servers::https::router, state::AppState,
        },
    };

    #[tokio::test]
    async fn test_get_user_by_id_success() -> TestResult {
        let user_id = Uuid::now_v7();
        let user = User {
            id: user_id.clone(),
            email: EmailAddress::new_unchecked("email@example.com"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut user_service = MockUserService::new();

        user_service
            .expect_get_user_by_id()
            .withf(move |id| *id == user.id)
            .returning(move |_| Ok(user.clone()));

        let state = AppState::new(user_service);

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
        let mut user_service = MockUserService::new();

        user_service
            .expect_get_user_by_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| Err(GetUserByIdError::UserNotFound(user_id.clone())));

        let state = AppState::new(user_service);

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
