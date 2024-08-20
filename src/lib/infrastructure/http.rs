//! HTTP Server module

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use axum_server::Handle;
use clap::Parser;
use tokio::signal;
use tracing::debug;

mod errors;
mod handlers;
mod open_api;
pub mod servers;
pub mod state;
mod templates;

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
    #[arg(long, env = "CERT_PATH")]
    pub cert_path: String,

    /// The path to the key file.
    #[arg(long, env = "KEY_PATH")]
    pub key_path: String,
}

/// The HTTP(S) server trait
#[async_trait]
pub trait Server {
    /// Runs the server.
    async fn run(self) -> Result<()>;
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
