//! HTTP application server

use std::net::{Ipv4Addr, SocketAddr, TcpListener};

use anyhow::{Context, Result};
use axum::{async_trait, extract::State, http::Uri, response::Redirect, routing::get, Router};
use axum_server::Handle;
use tracing::{debug, info};

use crate::infrastructure::http::{shutdown_signal, HttpServerConfig, Server};

/// The application's HTTP server
#[derive(Debug)]
pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    /// Returns a new HTTP server bound to the port specified in `config`.
    pub async fn new(config: HttpServerConfig) -> Result<Self> {
        let router = router(config.base_url);

        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.http_port));
        let listener = TcpListener::bind(address)
            .with_context(|| format!("failed to listen on {}", config.http_port))?;

        Ok(Self { router, listener })
    }
}

#[async_trait]
impl Server for HttpServer {
    /// Runs the HTTP server.
    #[mutants::skip]
    async fn run(self) -> Result<()> {
        debug!(
            "HTTP Server listening on {}",
            self.listener
                .local_addr()
                .context("failed to get local address")?
        );

        let handle = Handle::new();

        let server = axum_server::from_tcp(self.listener)
            .handle(handle.clone())
            .serve(self.router.into_make_service());

        tokio::select! {
            result = server => result.context("server error")?,
            _ = shutdown_signal(Some(handle)) => {
                info!("Shutting down HTTP server");
            }
        }

        Ok(())
    }
}

/// The HTTP handler
async fn http_handler(State(base_url): State<String>, uri: Uri) -> Redirect {
    debug!("redirecting to HTTPS: {}{}", base_url, uri.path());
    let uri = format!("{}{}", base_url, uri.path());

    Redirect::temporary(&uri)
}

/// Create the router for the HTTP server
pub fn router(base_url: String) -> Router {
    Router::new()
        .route("/*path", get(http_handler))
        .with_state(base_url)
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use testresult::TestResult;

    #[tokio::test]
    async fn test_http_server_redirect() -> TestResult {
        let base_url = "https://example.com".to_string();

        let router = super::router(base_url.clone());

        let response = TestServer::new(router)?.get("/abc/def").await;

        response.assert_status(StatusCode::TEMPORARY_REDIRECT);

        let location = response
            .headers()
            .get("location")
            .ok_or_else(|| anyhow!("missing location header"))?
            .to_str()?;

        assert_eq!(location, format!("{}{}", base_url, "/abc/def"));

        Ok(())
    }
}
