#![warn(
    missing_debug_implementations,
    rust_2018_idioms,
    missing_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs
)]

//! REST API for the application

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use rust_saas_starter::{
    domain::auth::services::user::UserService,
    infrastructure::{
        database::postgres::{DatabaseConnectionDetails, PostgresDatabase},
        http::{
            servers::{http::HttpServer, https::HttpsServer},
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

    let user_service = UserService::new(postgres);

    let http_config = args.server;

    let http_server = HttpServer::new(http_config.clone()).await?;
    let https_server = HttpsServer::new(user_service.clone(), http_config).await?;

    let http = tokio::spawn(http_server.run());
    let https = tokio::spawn(https_server.run());

    let _ = tokio::join!(http, https);

    Ok(())
}
