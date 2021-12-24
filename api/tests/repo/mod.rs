use mongodb::{Client, Database};
use uuid::Uuid;

mod company;

pub async fn new_db() -> anyhow::Result<Database> {
    let client = db_client().await?;
    let name = format!("test-{}", Uuid::new_v4());
    let db = client.database(&name);
    Ok(db)
}

pub async fn db_client() -> anyhow::Result<Client> {
    let client = Client::with_uri_str("mongodb://localhost:42781").await?;
    Ok(client)
}
