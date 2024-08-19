//! Create user handler

use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    domain::{
        auth::{
            models::user::NewUser, services::user::UserService, value_objects::password::Password,
        },
        comms::value_objects::email_address::EmailAddress,
    },
    infrastructure::http::{errors::ApiError, state::AppState},
};

/// Create user request body
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserBody {
    /// The new user's email address
    #[schema(example = "email@example.com")]
    email: String,

    /// The new user's password
    #[schema(example = "correcthorsebatterystaple")]
    password: String,
}

impl TryFrom<CreateUserBody> for NewUser {
    type Error = ApiError;

    fn try_from(body: CreateUserBody) -> Result<Self, Self::Error> {
        Ok(Self::new(
            Uuid::now_v7(),
            EmailAddress::new(&body.email)?,
            Password::new(&body.password)?,
            true,
        ))
    }
}

/// Create user response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserResponse {
    id: Uuid,

    #[schema(example = "email@example.com")]
    email: String,
}

/// Create a new user
#[utoipa::path(
    post,
    operation_id = "create_user",
    tag = "Auth",
    path = "/api/v1/users",
    request_body = CreateUserBody,
    responses(
        (status = StatusCode::CREATED, description = "User created", body = CreateUserResponse),
        (status = StatusCode::UNPROCESSABLE_ENTITY, description = "Unprocessable entity", body = ErrorResponse),
        (status = StatusCode::CONFLICT, description = "User already exists", body = ErrorResponse, example = json!({"message": "User with email \"email@example.com\" aleady exists"})),
    )
)]
pub async fn handler<U: UserService>(
    State(state): State<AppState<U>>,
    request: Result<Json<CreateUserBody>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateUserResponse>), ApiError> {
    let Json(request) = request?;
    let email = request.email.clone();

    let mut new_user: NewUser = request.try_into()?;

    new_user.email_confirmation_is_required(state.config.require_email_confirmation);

    let id = state
        .users
        .create_user(&new_user, &state.config.base_url)
        .await?;

    Ok((StatusCode::CREATED, Json(CreateUserResponse { id, email })))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use testresult::TestResult;
    use uuid::Uuid;

    use crate::{
        domain::{
            auth::{errors::CreateUserError, services::user::MockUserService},
            comms::value_objects::email_address::EmailAddress,
        },
        infrastructure::http::{
            errors::ErrorResponse,
            handlers::v1::auth::create_user::{CreateUserBody, CreateUserResponse},
            servers::https::router,
            state::test_state,
        },
    };

    impl CreateUserBody {
        /// Create a new `CreateUserBody` instance
        fn new(email: &str, password: &str) -> Self {
            Self {
                email: email.to_string(),
                password: password.to_string(),
            }
        }
    }

    #[tokio::test]
    async fn test_create_user_success() -> TestResult {
        let mut user_service = MockUserService::new();
        let user_id = Uuid::now_v7();

        let email = EmailAddress::new("email@example.com")?;
        let body = CreateUserBody::new(&email.to_string(), "correcthorsebatterystaple");

        user_service
            .expect_create_user()
            .withf(move |user, base_url| {
                user.email() == &email && base_url == "https://example.com"
            })
            .returning(move |_, _| Ok(user_id.clone()));

        let state = test_state(Some(user_service));

        let response = TestServer::new(router(state))?
            .post("/api/v1/users")
            .json(&body)
            .await;

        let json = response.json::<CreateUserResponse>();

        assert_eq!(response.status_code(), StatusCode::CREATED);
        assert_eq!(json.id, user_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_email_error() -> TestResult {
        let state = test_state(None);

        let response = TestServer::new(router(state))?
            .post("/api/v1/users")
            .json(&CreateUserBody::new(
                "not an email",
                "correcthorsebatterystaple",
            ))
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json.error, "Please provide a valid email address");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_password_error() -> TestResult {
        let state = test_state(None);

        let response = TestServer::new(router(state))?
            .post("/api/v1/users")
            .json(&CreateUserBody::new("email@example.com", "short"))
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json.error, "Password must be at least 8 characters long");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_user_duplicate_user() -> TestResult {
        let mut users = MockUserService::new();

        users.expect_create_user().returning(|_, _| {
            Err(CreateUserError::DuplicateUser {
                email: EmailAddress::new("email@example.com").expect("valid email"),
            })
        });

        let state = test_state(Some(users));

        let response = TestServer::new(router(state))?
            .post("/api/v1/users")
            .json(&CreateUserBody::new(
                "email@example.com",
                "correcthorsebatterystaple",
            ))
            .await;

        let json = response.json::<ErrorResponse>();

        assert_eq!(response.status_code(), StatusCode::CONFLICT);
        assert_eq!(
            json.error,
            "User with email \"email@example.com\" already exists"
        );

        Ok(())
    }
}
