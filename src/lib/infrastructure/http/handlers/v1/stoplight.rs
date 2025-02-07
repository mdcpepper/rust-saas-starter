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
    use testresult::TestResult;

    use crate::infrastructure::http::{servers::https::router, state::tests::test_state};

    #[tokio::test]
    async fn test_docs_handler() -> TestResult {
        let state = test_state(None, None);

        let response = TestServer::new(router(state))?
            .get("/api/v1")
            .content_type("text/html; charset=utf-8")
            .await;

        response.assert_status_ok();

        let raw_text = response.text();

        assert!(raw_text.contains("SaaS Starter API"));
        assert!(raw_text.contains("/api/v1/openapi.json"));

        Ok(())
    }
}
