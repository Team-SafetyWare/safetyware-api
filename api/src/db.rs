use crate::crockford;
use bson::Document;
use mongodb::{Client, Collection, Database, IndexModel};

pub const DB_NAME: &str = "sw";

pub mod coll {
    pub const COMPANY: &str = "company";
    pub const DEVICE: &str = "device";
    pub const GAS_READING: &str = "gas_reading";
    pub const INCIDENT: &str = "incident";
    pub const LOCATION_READING: &str = "location_reading";
    pub const PERSON: &str = "person";
    pub const TEAM: &str = "team";
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
    prepare_coll_gas_reading(db).await?;
    prepare_coll_incident(db).await?;
    prepare_coll_location_reading(db).await?;
    prepare_coll_person(db).await?;
    prepare_coll_team(db).await?;
    Ok(())
}

pub async fn prepare_coll_gas_reading(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection(coll::GAS_READING);
    create_simple_index(&collection, "person_id").await?;
    create_simple_index(&collection, "gas").await?;
    create_2dsphere_index(&collection, "location").await?;
    Ok(())
}

pub async fn prepare_coll_incident(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection(coll::INCIDENT);
    create_simple_index(&collection, "person_id").await?;
    create_2dsphere_index(&collection, "location").await?;
    Ok(())
}

pub async fn prepare_coll_location_reading(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection(coll::LOCATION_READING);
    create_simple_index(&collection, "person_id").await?;
    create_2dsphere_index(&collection, "location").await?;
    Ok(())
}

pub async fn prepare_coll_person(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection(coll::PERSON);
    create_simple_index(&collection, "company_id").await?;
    Ok(())
}

pub async fn prepare_coll_team(db: &Database) -> anyhow::Result<()> {
    let collection = db.collection(coll::TEAM);
    create_simple_index(&collection, "people.person_id").await?;
    Ok(())
}

pub async fn create_simple_index(
    collection: &Collection<Document>,
    field: &str,
) -> anyhow::Result<()> {
    collection
        .create_index(
            IndexModel::builder().keys(bson::doc! { field: 1 }).build(),
            None,
        )
        .await?;
    Ok(())
}

pub async fn create_2dsphere_index(
    collection: &Collection<Document>,
    field: &str,
) -> anyhow::Result<()> {
    collection
        .create_index(
            IndexModel::builder()
                .keys(bson::doc! { field: "2dsphere" })
                .build(),
            None,
        )
        .await?;
    Ok(())
}
