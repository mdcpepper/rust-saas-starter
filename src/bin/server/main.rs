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
    domain::{
        auth::users::UserServiceImpl, communication::email_addresses::EmailAddressServiceImpl,
    },
    infrastructure::{
        db::postgres::{DatabaseConnectionDetails, PostgresDatabase},
        email::smtp::{SMTPConfig, SMTPMailer},
        http::{
            servers::{http::HttpServer, https::HttpsServer},
            state::{AppConfig, AppState},
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

    /// SMTP server configuration
    #[clap(flatten)]
    pub smtp: SMTPConfig,
}

#[mutants::skip]
#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load environment: {}", e);

        return Err(e.into());
    }

    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let postgres = Arc::new(PostgresDatabase::new(&args.db.connection_string).await?);
    let mailer = Arc::new(SMTPMailer::new(args.smtp));

    let config = AppConfig {
        base_url: args.server.base_url.clone(),
    };

    let state = AppState {
        config,
        start_time: Utc::now(),
        users: Arc::new(UserServiceImpl::new(postgres.clone())),
        email_addresses: Arc::new(EmailAddressServiceImpl::new(postgres, mailer)),
    };

    let http_port = args.server.http_port;
    let https_port = args.server.https_port;

    let _ = tokio::join!(
        tokio::spawn(
            HttpServer::new(
                SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), http_port),
                &args.server.base_url,
            )
            .await?
            .run()
        ),
        tokio::spawn(
            HttpServer::new(
                SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), http_port),
                &args.server.base_url,
            )
            .await?
            .run()
        ),
        tokio::spawn(
            HttpsServer::new(
                SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), https_port),
                &args.server.cert_path,
                &args.server.key_path,
                state.clone(),
            )
            .await?
            .run()
        ),
        tokio::spawn(
            HttpsServer::new(
                SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), https_port),
                &args.server.cert_path,
                &args.server.key_path,
                state,
            )
            .await?
            .run()
        ),
    );

    Ok(())
}
