use crate::db::coll;
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGasReading {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub gas: String,
    pub density: f64,
    pub location: DbLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbLocation {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasReading {
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub gas: String,
    pub density: f64,
    pub coordinates: Vec<f64>,
}

impl From<DbGasReading> for GasReading {
    fn from(value: DbGasReading) -> Self {
        Self {
            timestamp: value.timestamp,
            person_id: value.person_id,
            gas: value.gas,
            density: value.density,
            coordinates: value.location.coordinates,
        }
    }
}

impl From<GasReading> for DbGasReading {
    fn from(value: GasReading) -> Self {
        Self {
            timestamp: value.timestamp,
            person_id: value.person_id,
            gas: value.gas,
            density: value.density,
            location: DbLocation {
                r#type: "Point".to_string(),
                coordinates: value.coordinates,
            },
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct GasReadingFilter {
    pub person_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait GasReadingRepo {
    async fn insert_many(&self, gas_readings: &[GasReading]) -> anyhow::Result<()>;

    async fn find(
        &self,
        filter: GasReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<GasReading>>>;
}

#[derive(Debug, Clone)]
pub struct MongoGasReadingRepo {
    pub db: Database,
}

impl MongoGasReadingRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<DbGasReading> {
        self.db.collection(coll::GAS_READING)
    }
}

#[async_trait::async_trait]
impl GasReadingRepo for MongoGasReadingRepo {
    async fn insert_many(&self, gas_readings: &[GasReading]) -> anyhow::Result<()> {
        let db_readings: Vec<DbGasReading> = gas_readings
            .to_vec()
            .into_iter()
            .map(|r| r.into())
            .collect();
        self.collection().insert_many(db_readings, None).await?;
        Ok(())
    }

    async fn find(
        &self,
        filter: GasReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<GasReading>>> {
        let mut mongo_filter = Document::new();
        if let Some(person_ids) = filter.person_ids {
            mongo_filter.insert("person_id", bson::doc! { "$in": person_ids });
        }
        if let Some(min_timestamp) = filter.min_timestamp {
            mongo_filter
                .entry("timestamp".to_string())
                .or_insert(bson::doc! {}.into())
                .as_document_mut()
                .unwrap()
                .insert("$gte", min_timestamp);
        }
        if let Some(max_timestamp) = filter.max_timestamp {
            mongo_filter
                .entry("timestamp".to_string())
                .or_insert(bson::doc! {}.into())
                .as_document_mut()
                .unwrap()
                .insert("$lt", max_timestamp);
        }
        let cursor = self.collection().find(mongo_filter, None).await?;
        let stream = cursor.map_ok(Into::into).map_err(|e| e.into());
        Ok(Box::new(stream))
    }
}
