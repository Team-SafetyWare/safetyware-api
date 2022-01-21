use crate::crockford;
use bson::Document;
use mongodb::{Client, Database, IndexModel};

pub const DB_NAME: &str = "sw";

pub mod coll {
    pub const COMPANY: &str = "company";
    pub const LOCATION_READING: &str = "location_reading";
    pub const PERSON: &str = "person";
    pub const USER_ACCOUNT: &str = "user_account";
}

pub mod err_code {
    pub const NAMESPACE_EXISTS: i32 = 48;
}

pub async fn connect(db_uri: &str) -> anyhow::Result<Database> {
    let db = Client::with_uri_str(db_uri).await?.database(DB_NAME);
    Ok(db)
}

pub async fn connect_and_prepare(db_uri: &str) -> anyhow::Result<Database> {
    let db = connect(db_uri).await?;
    prepare(&db).await?;
    Ok(db)
}

pub async fn test_connection(db: &Database) -> anyhow::Result<()> {
    let nonexistent = crockford::random_id();
    db.collection::<Document>(&nonexistent)
        .find_one(None, None)
        .await?;
    Ok(())
}

pub async fn prepare(db: &Database) -> anyhow::Result<()> {
    prepare_coll_person(db).await?;
    prepare_coll_location_reading(db).await?;
    Ok(())
}

pub async fn prepare_coll_person(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection::<Document>(coll::PERSON);
    collection
        .create_index(
            IndexModel::builder()
                .keys(bson::doc! { "company_id": 1 })
                .build(),
            None,
        )
        .await?;
    Ok(())
}

pub async fn prepare_coll_location_reading(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection::<Document>(coll::LOCATION_READING);
    collection
        .create_index(
            IndexModel::builder()
                .keys(bson::doc! { "person_id": 1 })
                .build(),
            None,
        )
        .await?;
    collection
        .create_index(
            IndexModel::builder()
                .keys(bson::doc! { "location": "2dsphere" })
                .build(),
            None,
        )
        .await?;
    Ok(())
}
