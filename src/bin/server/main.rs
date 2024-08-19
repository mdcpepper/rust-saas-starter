#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

//! REST API for the application

use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use rust_saas_starter::{
    domain::auth::services::user::{UserManagement, UserService},
    infrastructure::{
        database::postgres::{DatabaseConnectionDetails, PostgresDatabase},
        http::{
            servers::{http::HttpServer, https::HttpsServer},
            state::AppState,
            HttpServerConfig, Server,
        },
    },
};

/// Command-line arguments / environment variables
#[derive(Debug, Parser)]
pub struct Args {
    /// The HTTP server configuration
    #[clap(flatten)]
    pub server: HttpServerConfig,

    /// The database connection details
    #[clap(flatten)]
    pub db: DatabaseConnectionDetails,
}

#[mutants::skip]
#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load environment: {}", e);

        return Err(e.into());
    }

    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let postgres = Arc::new(PostgresDatabase::new(&args.db.connection_string).await?);

    let (ipv4_http_server, ipv6_http_server) =
        get_http_servers(args.server.http_port, &args.server.base_url).await?;

    let state = AppState {
        start_time: Utc::now(),
        users: Arc::new(UserService::new(postgres)),
    };

    let (ipv4_https_server, ipv6_https_server) = get_https_servers(
        args.server.https_port,
        &args.server.cert_path,
        &args.server.key_path,
        state,
    )
    .await?;

    let _ = tokio::join!(
        tokio::spawn(ipv4_http_server.run()),
        tokio::spawn(ipv6_http_server.run()),
        tokio::spawn(ipv4_https_server.run()),
        tokio::spawn(ipv6_https_server.run()),
    );

    Ok(())
}

async fn get_http_servers(http_port: u16, base_url: &str) -> Result<(HttpServer, HttpServer)> {
    Ok((
        HttpServer::new(
            SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), http_port),
            base_url,
        )
        .await?,
        HttpServer::new(
            SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), http_port),
            base_url,
        )
        .await?,
    ))
}

async fn get_https_servers(
    port: u16,
    cert_path: &str,
    key_path: &str,
    state: AppState<impl UserManagement>,
) -> Result<(HttpsServer, HttpsServer)> {
    Ok((
        HttpsServer::new(
            SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port),
            cert_path,
            key_path,
            state.clone(),
        )
        .await?,
        HttpsServer::new(
            SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port),
            cert_path,
            key_path,
            state,
        )
        .await?,
    ))
}
