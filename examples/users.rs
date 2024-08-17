use anyhow::Result;
use clap::Parser;
use rust_saas_starter::{
    domain::auth::{
        models::user::CreateUserRequest, repositories::user::UserRepository,
        value_objects::email_address::EmailAddress,
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
    dotenvy::dotenv().ok();

    let args = Args::parse();

    let database = PostgresDatabase::new_with_url(&args.db.connection_string)
        .await
        .expect("Failed to connect to the database");

    let create_user =
        CreateUserRequest::new(Uuid::now_v7(), EmailAddress::new("email@example.com")?);

    let uuid = database.create_user(&create_user).await?;

    println!("Created user with UUID: {:#?}", uuid);

    Ok(())
}
