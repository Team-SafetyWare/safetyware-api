use crate::crockford;
use bson::Document;
use mongodb::{Client, Database};

pub const DB_NAME: &str = "sw";

pub async fn connect(db_uri: &str) -> anyhow::Result<Database> {
    let db = Client::with_uri_str(db_uri).await?.database(DB_NAME);
    Ok(db)
}

pub async fn test_connection(db: &Database) -> anyhow::Result<()> {
    let nonexistent = crockford::random_id();
    db.collection::<Document>(&nonexistent)
        .find_one(None, None)
        .await?;
    Ok(())
}
