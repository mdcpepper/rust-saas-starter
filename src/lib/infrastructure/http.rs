//! HTTP Server

use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use axum::{extract::Request, Router};
use axum_server::Handle;
use chrono::Utc;
use clap::Parser;
use handlers::v1;
use state::AppState;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::domain::auth::services::user::UserManagement;

mod errors;
mod handlers;
mod open_api;
mod state;

/// Configuration for the HTTP server.
#[derive(Debug, Clone, PartialEq, Eq, Parser)]
pub struct HttpServerConfig {
    /// The port to listen on
    #[arg(short, long, env = "HTTP_PORT", default_value = "3000")]
    pub port: u16,
}

/// The application's HTTP server
#[derive(Debug)]
pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    /// Returns a new HTTP server bound to the port specified in `config`.
    pub async fn new(
        user_service: impl UserManagement,
        config: HttpServerConfig,
    ) -> anyhow::Result<Self> {
        let state = AppState {
            start_time: Utc::now(),
            users: Arc::new(user_service),
        };

        let router = router(state);

        let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
        let listener = TcpListener::bind(address)
            .with_context(|| format!("failed to listen on {}", config.port))?;

        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    #[mutants::skip]
    pub async fn run(self) -> anyhow::Result<()> {
        debug!("listening on {}", self.listener.local_addr().unwrap());

        let handle = Handle::new();

        shutdown_signal(Some(handle.clone())).await;

        axum_server::from_tcp(self.listener)
            .handle(handle)
            .serve(self.router.into_make_service())
            .await?;

        Ok(())
    }
}

/// Create the application's router
pub fn router<US: UserManagement>(state: AppState<US>) -> Router {
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        let uri = request.uri().to_string();
        tracing::info_span!("http_request", method = ?request.method(), uri)
    });

    Router::new()
        .layer(trace_layer)
        .nest("/api/v1", v1::router())
        .with_state(state)
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
