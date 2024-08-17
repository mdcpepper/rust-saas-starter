use anyhow::{Context, Result};
use clap::Parser;
use rust_saas_starter::{
    domain::auth::{
        models::user::CreateUserRequest,
        repositories::user::UserRepository,
        value_objects::{email_address::EmailAddress, password::Password},
    },
    infrastructure::database::postgres::{DatabaseConnectionDetails, PostgresDatabase},
};
use uuid::Uuid;

#[derive(Parser)]
pub struct Args {
    #[clap(flatten)]
    pub db: DatabaseConnectionDetails,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Failed to load .env file: {}", e);
        return Err(e.into());
    }

    let args = Args::parse();

    let database = PostgresDatabase::new_with_url(&args.db.connection_string)
        .await
        .context("Failed to connect to the database")?;

    let create_user = CreateUserRequest::new(
        Uuid::now_v7(),
        EmailAddress::new("email@example.com")?,
        Password::new("password")?,
    );

    let uuid = database.create_user(&create_user).await?;

    println!("Created user with UUID: {:#?}", uuid);

    Ok(())
}
