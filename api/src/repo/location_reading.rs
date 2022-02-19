use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, InsertOpt};
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbLocationReading {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub location: DbLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbLocation {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReading {
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub coordinates: Vec<f64>,
}

impl From<DbLocationReading> for LocationReading {
    fn from(value: DbLocationReading) -> Self {
        Self {
            person_id: value.person_id,
            timestamp: value.timestamp,
            coordinates: value.location.coordinates,
        }
    }
}

impl From<LocationReading> for DbLocationReading {
    fn from(value: LocationReading) -> Self {
        Self {
            person_id: value.person_id,
            timestamp: value.timestamp,
            location: DbLocation {
                r#type: "Point".to_string(),
                coordinates: value.coordinates,
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct LocationReadingFilter {
    pub person_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait LocationReadingRepo {
    async fn insert_many(&self, location_readings: Vec<LocationReading>) -> anyhow::Result<()>;

    async fn find(
        &self,
        filter: LocationReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>>;
}

#[derive(Debug, Clone)]
pub struct MongoLocationReadingRepo {
    pub db: Database,
}

impl MongoLocationReadingRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<DbLocationReading> {
        self.db.collection(coll::LOCATION_READING)
    }
}

#[async_trait::async_trait]
impl LocationReadingRepo for MongoLocationReadingRepo {
    async fn insert_many(&self, location_readings: Vec<LocationReading>) -> anyhow::Result<()> {
        let db_readings: Vec<DbLocationReading> =
            location_readings.into_iter().map(|r| r.into()).collect();
        self.collection().insert_many(db_readings, None).await?;
        Ok(())
    }

    async fn find(
        &self,
        filter: LocationReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>> {
        let mut mongo_filter = Document::new();
        mongo_filter.insert_opt("person_id", filter::one_of(filter.person_ids));
        mongo_filter.insert_opt(
            "timestamp",
            filter::clamp(filter.min_timestamp, filter.max_timestamp),
        );
        self.collection().find_stream(mongo_filter, None).await
    }
}
