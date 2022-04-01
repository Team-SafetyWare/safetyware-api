use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, InsertOpt};
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use mongodb::options::FindOptions;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGasReading {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub gas: String,
    pub density: f64,
    pub density_units: String,
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
    pub density_units: String,
    pub coordinates: Vec<f64>,
}

impl From<DbGasReading> for GasReading {
    fn from(value: DbGasReading) -> Self {
        Self {
            timestamp: value.timestamp,
            person_id: value.person_id,
            gas: value.gas,
            density: value.density,
            density_units: value.density_units,
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
            density_units: value.density_units,
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
    async fn insert_many(&self, gas_readings: Vec<GasReading>) -> anyhow::Result<()>;

    async fn find(
        &self,
        filter: GasReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<GasReading>>>;
}

pub type DynGasReadingRepo = dyn GasReadingRepo + Send + Sync + 'static;

pub type ArcGasReadingRepo = Arc<DynGasReadingRepo>;

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
    async fn insert_many(&self, gas_readings: Vec<GasReading>) -> anyhow::Result<()> {
        let db_readings: Vec<DbGasReading> = gas_readings.into_iter().map(Into::into).collect();
        self.collection().insert_many(db_readings, None).await?;
        Ok(())
    }

    async fn find(
        &self,
        filter: GasReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<GasReading>>> {
        let mut mongo_filter = Document::new();
        mongo_filter.insert("hidden", filter::not_true());
        mongo_filter.insert("density", filter::not(0));
        mongo_filter.insert_opt("person_id", filter::one_of(filter.person_ids));
        mongo_filter.insert_opt(
            "timestamp",
            filter::clamp(filter.min_timestamp, filter.max_timestamp),
        );
        self.collection()
            .find_stream(
                mongo_filter,
                FindOptions::builder()
                    .sort(bson::doc! {"timestamp": 1})
                    .build(),
            )
            .await
    }
}

impl From<MongoGasReadingRepo> for ArcGasReadingRepo {
    fn from(value: MongoGasReadingRepo) -> Self {
        Arc::new(value)
    }
}
