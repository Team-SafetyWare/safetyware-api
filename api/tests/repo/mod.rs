use api::crockford;
use mongodb::{Client, Database};

mod mongo_op;

pub async fn new_db() -> anyhow::Result<Database> {
    let client = db_client().await?;
    let name = format!("test-{}", crockford::random_id());
    let db = client.database(&name);
    Ok(db)
}

pub async fn db_client() -> anyhow::Result<Client> {
    let client = Client::with_uri_str("mongodb://localhost:42781").await?;
    Ok(client)
}
