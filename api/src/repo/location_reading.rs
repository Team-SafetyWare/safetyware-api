use crate::db::coll;
use crate::repo::op::Find;
use crate::repo::{mongo_op, ItemStream};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReading {
    pub timestamp: String,
    pub person_id: String,
    pub coordinates: Vec<f64>,
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

    pub fn collection(&self) -> Collection<LocationReading> {
        self.db.collection(coll::LOCATION_READING)
    }
}

impl LocationReadingRepo for MongoLocationReadingRepo {}

#[async_trait::async_trait]
impl Find<LocationReading> for MongoLocationReadingRepo {
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>> {
        mongo_op::find(&self.collection()).await
    }
}
