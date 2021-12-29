use crate::repo::op::Find;
use crate::repo::{mongo_op, ItemStream};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReading {
    pub timestamp: String,
    pub person_id: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoLocationReading {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub metadata: Metadata,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub person_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub coordinates: Vec<f64>,
}

impl From<MongoLocationReading> for LocationReading {
    fn from(value: MongoLocationReading) -> Self {
        Self {
            person_id: value.metadata.person_id,
            timestamp: value.timestamp.to_string(),
            coordinates: value.location.coordinates,
        }
    }
}

#[async_trait::async_trait]
pub trait LocationReadingRepo: Find<LocationReading> {}

#[derive(Debug, Clone)]
pub struct MongoLocationReadingRepo {
    pub db: Database,
}

impl MongoLocationReadingRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<MongoLocationReading> {
        self.db.collection("location_reading")
    }
}

impl LocationReadingRepo for MongoLocationReadingRepo {}

#[async_trait::async_trait]
impl Find<LocationReading> for MongoLocationReadingRepo {
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>> {
        let mongo_stream = mongo_op::find(&self.collection()).await?;
        let stream = mongo_stream.map_ok(Into::into);
        Ok(Box::new(stream))
    }
}