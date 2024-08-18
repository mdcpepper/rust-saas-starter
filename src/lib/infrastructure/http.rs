//! HTTP Server

use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use axum::{
    extract::{Request, State},
    http::Uri,
    response::Redirect,
    routing::get,
    Router,
};
use axum_server::{tls_rustls::RustlsConfig, Handle};
use chrono::Utc;
use clap::Parser;
use handlers::v1;
use state::AppState;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{debug, info, info_span};

use crate::domain::auth::services::user::UserManagement;

mod errors;
mod handlers;
mod open_api;
mod state;

/// Configuration for the HTTP server.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub struct HttpServerConfig {
    /// The port the HTTP server should listen on.
    #[arg(long, env = "HTTP_PORT", default_value = "3000")]
    pub http_port: u16,

    /// The port the HTTPS server should listen on.
    #[arg(long, env = "HTTPS_PORT", default_value = "3443")]
    pub https_port: u16,

    /// The base URL of the server.
    #[arg(long, env = "BASE_URL", default_value = "https://localhost:3443")]
    pub base_url: String,

    /// The path to the certificate file.
    #[arg(long, env = "CERT_PATH", default_value = "certs/cert.pem")]
    pub cert_path: String,

    /// The path to the key file.
    #[arg(long, env = "KEY_PATH", default_value = "certs/key.pem")]
    pub key_path: String,
}

/// The HTTP(S) server trait
#[async_trait]
pub trait Server {
    /// Runs the server.
    async fn run(self) -> Result<()>;
}

/// The application's HTTP server
#[derive(Debug)]
pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    /// Returns a new HTTP server bound to the port specified in `config`.
    pub async fn new(config: HttpServerConfig) -> Result<Self> {
        let router = Router::new()
            .route("/*path", get(http_handler))
            .with_state(config.base_url);

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

/// The application's HTTPS server
#[derive(Debug)]
pub struct HttpsServer {
    router: Router,
    address: SocketAddr,
    tls_config: RustlsConfig,
}

impl HttpsServer {
    /// Returns a new HTTPS server bound to the port specified in `config`.
    pub async fn new(user_service: impl UserManagement, config: HttpServerConfig) -> Result<Self> {
        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.https_port));

        let state = AppState {
            start_time: Utc::now(),
            users: Arc::new(user_service),
        };

        let tls_config = RustlsConfig::from_pem_file(config.cert_path, config.key_path)
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

/// Create the application's router
pub fn router<US: UserManagement>(state: AppState<US>) -> Router {
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        let uri = request.uri().to_string();
        info_span!("http_request", method = ?request.method(), uri)
    });

    Router::new()
        .layer(trace_layer)
        .nest("/api/v1", v1::router())
        .with_state(state)
}

/// The HTTP handler
async fn http_handler(State(base_url): State<String>, uri: Uri) -> Redirect {
    debug!("redirecting to HTTPS: {}{}", base_url, uri.path());
    let uri = format!("{}{}", base_url, uri.path());

    Redirect::temporary(&uri)
}

#[mutants::skip]
async fn shutdown_signal(handle: Option<Handle>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    if let Some(handle) = handle {
        debug!("shutting down gracefully");
        handle.graceful_shutdown(Some(Duration::from_secs(10)));
    }
}
