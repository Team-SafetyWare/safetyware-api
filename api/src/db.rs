use crate::crockford;
use bson::Document;
use mongodb::error::{Error, ErrorKind};
use mongodb::options::CreateCollectionOptions;
use mongodb::{Client, Database};

pub const DB_NAME: &str = "sw";

pub mod coll {
    pub const COMPANY: &str = "company";
    pub const LOCATION_READING: &str = "location_reading";
    pub const PERSON: &str = "person";
}

pub mod err_code {
    pub const NAMESPACE_EXISTS: i32 = 48;
}

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

pub async fn prepare(db: &Database) -> anyhow::Result<()> {
    prepare_location_reading_collection(db).await?;
    Ok(())
}

async fn prepare_location_reading_collection(db: &Database) -> anyhow::Result<()> {
    let name = coll::LOCATION_READING;
    let res = db
        .create_collection(
            name,
            Some(
                CreateCollectionOptions::builder()
                    .timeseries(Some(
                        // There seems to be no better way to create a TimeseriesOptions struct.
                        serde_json::from_str(r#"{"timeField":"timestamp","metaField":"metadata"}"#)
                            .unwrap(),
                    ))
                    .build(),
            ),
        )
        .await;
    if let Err(e) = res {
        if command_err_code(&e) != Some(err_code::NAMESPACE_EXISTS) {
            Err(e.into())
        } else {
            log_collection_exists(name);
            Ok(())
        }
    } else {
        log_collection_created(name);
        Ok(())
    }
}

fn command_err_code(error: &Error) -> Option<i32> {
    if let ErrorKind::Command(e) = &*error.kind {
        Some(e.code)
    } else {
        None
    }
}

fn log_collection_exists(name: &str) {
    log::info!("MongoDB collection '{}' already exists", name);
}

fn log_collection_created(name: &str) {
    log::info!("MongoDB collection '{}' created", name);
}
