//! HTTPS application server

use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{async_trait, extract::Request, Router};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span};

use crate::{
    domain::auth::services::{email_address::EmailAddressService, user::UserService},
    infrastructure::http::{handlers::v1, shutdown_signal, state::AppState, Server},
};

/// The application's HTTPS server
#[derive(Debug)]
pub struct HttpsServer {
    router: Router,
    address: SocketAddr,
    tls_config: RustlsConfig,
}

impl HttpsServer {
    /// Returns a new HTTPS server bound to the port specified in `config`.
    pub async fn new(
        address: SocketAddr,
        cert_path: &str,
        key_path: &str,
        state: AppState<impl UserService, impl EmailAddressService>,
    ) -> Result<Self> {
        let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .context("failed to load TLS config")?;

        let router = router(state);

        Ok(Self {
            router,
            address,
            tls_config,
        })
    }
}

#[async_trait]
impl Server for HttpsServer {
    async fn run(self) -> Result<()> {
        debug!("HTTPS Server listening on {}", self.address);

        let handle = Handle::new();

        let server = axum_server::bind_rustls(self.address, self.tls_config)
            .handle(handle.clone())
            .serve(self.router.into_make_service());

        tokio::select! {
            result = server => result.context("server error")?,
            _ = shutdown_signal(Some(handle)) => {
                info!("Shutting down HTTPS server");
            }
        }

        Ok(())
    }
}

/// Create the router for the HTTPS server
pub fn router<U: UserService, E: EmailAddressService>(state: AppState<U, E>) -> Router {
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        let uri = request.uri().to_string();
        info_span!("http_request", method = ?request.method(), uri)
    });

    Router::new()
        .layer(trace_layer)
        .nest("/api/v1", v1::router())
        .with_state(state)
}
