//! API documentation.

use axum::response::Html;

/// Stoplight API documentation.
pub async fn handler() -> Html<String> {
    Html(
        r#"
<html lang="en">
<head>
    <title>SaaS Starter API</title>
    <script src="https://unpkg.com/@stoplight/elements/web-components.min.js"></script>
    <link rel="stylesheet" href="https://unpkg.com/@stoplight/elements/styles.min.css">
</head>
<body>
    <main role="main">
        <elements-api apiDescriptionUrl="/api/v1/openapi.json" router="hash" />
    </main>
</body>
</html>
"#
        .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;

    use crate::{
        domain::auth::{repositories::user::MockUserRepository, services::user::UserService},
        infrastructure::http::{
            router,
            state::{get_test_state, AppState},
        },
    };

    #[tokio::test]
    async fn test_docs_handler() {
        let user_repo = MockUserRepository::new();
        let state: AppState<UserService<MockUserRepository>> = get_test_state(user_repo);

        let response = TestServer::new(router(state))
            .unwrap()
            .get("/api/v1")
            .content_type("text/html; charset=utf-8")
            .await;

        response.assert_status_ok();

        let raw_text = response.text();

        assert!(raw_text.contains("SaaS Starter API"));
        assert!(raw_text.contains("/api/v1/openapi.json"));
    }
}
