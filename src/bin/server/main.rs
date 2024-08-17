use std::sync::Arc;

use clap::Parser;
use rust_saas_starter::{
    domain::auth::services::user::UserService,
    infrastructure::{
        database::postgres::{DatabaseConnectionDetails, PostgresDatabase},
        http::{HttpServer, HttpServerConfig},
    },
};

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    pub server: HttpServerConfig,

    #[clap(flatten)]
    pub db: DatabaseConnectionDetails,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load environment: {}", e);

        return Err(e.into());
    }

    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let postgres = Arc::new(PostgresDatabase::new(&args.db.connection_string).await?);

    let user_service = UserService::new(postgres);

    let http_config = HttpServerConfig {
        port: args.server.port,
    };

    let http_server = HttpServer::new(user_service, http_config).await?;

    http_server.run().await
}
