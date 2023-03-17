pub mod pool_batch;
pub mod pool_tx;

use lazy_static::lazy_static;
use mongodb::{options::ClientOptions, Client, Database};

use tokio::sync::OnceCell;

lazy_static! {
    static ref DB_CLIENT: OnceCell<Client> = OnceCell::new();
}

async fn connect_client() -> Client {
    let db_host = std::env::var("DB_HOST").unwrap_or(String::from("localhost"));
    let db_port = std::env::var("DB_PORT").unwrap_or(String::from("27017"));
    let db_username = std::env::var("DB_USERNAME").unwrap_or(String::from("root"));
    let db_password = std::env::var("DB_PASSWORD").unwrap_or(String::from("123456"));

    let mut client_options = ClientOptions::parse(format!(
        "mongodb://{}:{}@{}:{}",
        db_username, db_password, db_host, db_port
    ))
    .await
    .unwrap();
    client_options.app_name = Some("bundler-client".to_string());

    Client::with_options(client_options).unwrap()
}

pub async fn get_database() -> Database {
    let client = DB_CLIENT.get_or_init(connect_client).await;
    client.database("zkprover_bundler")
}
