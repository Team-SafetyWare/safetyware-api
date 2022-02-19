use crate::db::coll;
use crate::repo::team::{MongoTeamRepo, TeamRepo};
use crate::repo::{filter_util, ItemStream};
use bson::Document;
use chrono::{DateTime, Utc};
use futures_util::{stream, StreamExt, TryStreamExt};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
    pub team_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait LocationReadingRepo {
    async fn insert_many(&self, location_readings: &[LocationReading]) -> anyhow::Result<()>;

    async fn find(
        &self,
        filter: LocationReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>>;
}

#[derive(Debug, Clone)]
pub struct MongoLocationReadingRepo {
    pub db: Database,
    pub team_repo: MongoTeamRepo,
}

impl MongoLocationReadingRepo {
    pub fn new(db: Database) -> Self {
        Self {
            team_repo: MongoTeamRepo::new(db.clone()),
            db,
        }
    }

    pub fn collection(&self) -> Collection<DbLocationReading> {
        self.db.collection(coll::LOCATION_READING)
    }
}

#[async_trait::async_trait]
impl LocationReadingRepo for MongoLocationReadingRepo {
    async fn insert_many(&self, location_readings: &[LocationReading]) -> anyhow::Result<()> {
        let db_readings: Vec<DbLocationReading> = location_readings
            .to_vec()
            .into_iter()
            .map(|r| r.into())
            .collect();
        self.collection().insert_many(db_readings, None).await?;
        Ok(())
    }

    async fn find(
        &self,
        filter: LocationReadingFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<LocationReading>>> {
        let mut mongo_filter = Document::new();
        if filter.person_ids.is_some() || filter.team_ids.is_some() {
            let person_ids: Vec<String> = stream::iter(filter.team_ids.unwrap_or_default())
                .map(|team_id| async move { self.team_repo.find_people(&team_id).await })
                .buffered(10)
                .try_flatten()
                .map_ok(|tp| tp.person_id)
                .chain(stream::iter(
                    filter.person_ids.unwrap_or_default().into_iter().map(Ok),
                ))
                .try_collect::<HashSet<String>>()
                .await?
                .into_iter()
                .collect();
            mongo_filter.insert("person_id", bson::doc! { "$in": person_ids });
        }
        mongo_filter.insert(
            "timestamp",
            filter_util::clamp_timestamp(filter.min_timestamp, filter.max_timestamp),
        );
        let cursor = self.collection().find(mongo_filter, None).await?;
        let stream = cursor.map_ok(Into::into).map_err(|e| e.into());
        Ok(Box::new(stream))
    }
}
