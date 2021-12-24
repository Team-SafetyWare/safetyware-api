use bson::Document;
use mongodb::{Client, Database};
use uuid::Uuid;

pub const DB_NAME: &str = "sw";

pub async fn connect(db_uri: &str) -> anyhow::Result<Database> {
    let db = Client::with_uri_str(db_uri).await?.database(DB_NAME);
    Ok(db)
}

pub async fn test_connection(db: &Database) -> anyhow::Result<()> {
    let nonexistent = Uuid::new_v4().to_string();
    db.collection::<Document>(&nonexistent)
        .find_one(None, None)
        .await?;
    Ok(())
}
