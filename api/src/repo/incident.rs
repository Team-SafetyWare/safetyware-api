use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, FromDeletedCount, FromMatchedCount, InsertOpt};
use crate::repo::{DeleteResult, ItemStream, ReplaceResult};
use bson::Document;
use chrono::{DateTime, Utc};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbIncident {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub location: DbLocation,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbLocation {
    pub r#type: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub person_id: String,
    pub coordinates: Vec<f64>,
    pub r#type: String,
}

impl From<DbIncident> for Incident {
    fn from(value: DbIncident) -> Self {
        Self {
            id: value.id,
            timestamp: value.timestamp,
            person_id: value.person_id,
            coordinates: value.location.coordinates,
            r#type: value.r#type,
        }
    }
}

impl From<Incident> for DbIncident {
    fn from(value: Incident) -> Self {
        Self {
            id: value.id,
            timestamp: value.timestamp,
            person_id: value.person_id,
            location: DbLocation {
                r#type: "Point".to_string(),
                coordinates: value.coordinates,
            },
            r#type: value.r#type,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct IncidentFilter {
    pub person_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait IncidentRepo {
    async fn insert_one(&self, incident: Incident) -> anyhow::Result<()>;
    async fn insert_many(&self, incidents: Vec<Incident>) -> anyhow::Result<()>;
    async fn replace_one(&self, incident: Incident) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Incident>>;
    async fn find(&self, filter: IncidentFilter) -> anyhow::Result<Box<dyn ItemStream<Incident>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

pub type DynIncidentRepo = dyn IncidentRepo + Send + Sync + 'static;

pub type ArcIncidentRepo = Arc<DynIncidentRepo>;

#[derive(Debug, Clone)]
pub struct MongoIncidentRepo {
    pub db: Database,
}

impl MongoIncidentRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<DbIncident> {
        self.db.collection(coll::INCIDENT)
    }
}

#[async_trait::async_trait]
impl IncidentRepo for MongoIncidentRepo {
    async fn insert_one(&self, incident: Incident) -> anyhow::Result<()> {
        let db_incident: DbIncident = incident.into();
        self.collection().insert_one(db_incident, None).await?;
        Ok(())
    }

    async fn insert_many(&self, incidents: Vec<Incident>) -> anyhow::Result<()> {
        let db_incidents: Vec<DbIncident> = incidents.into_iter().map(Into::into).collect();
        self.collection().insert_many(db_incidents, None).await?;
        Ok(())
    }

    async fn replace_one(&self, incident: Incident) -> ReplaceResult {
        let db_incident: DbIncident = incident.into();
        let res = self
            .collection()
            .replace_one(bson::doc! {"_id": &db_incident.id}, db_incident, None)
            .await
            .map_err(anyhow::Error::from)?;
        ReplaceResult::from_matched_count(res.matched_count)
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Incident>> {
        Ok(self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?
            .map(Into::into))
    }

    async fn find(&self, filter: IncidentFilter) -> anyhow::Result<Box<dyn ItemStream<Incident>>> {
        let mut mongo_filter = Document::new();
        mongo_filter.insert_opt("person_id", filter::one_of(filter.person_ids));
        mongo_filter.insert_opt(
            "timestamp",
            filter::clamp(filter.min_timestamp, filter.max_timestamp),
        );
        self.collection().find_stream(mongo_filter, None).await
    }

    async fn delete_one(&self, id: &str) -> DeleteResult {
        let res = self
            .collection()
            .delete_one(bson::doc! {"_id": id}, None)
            .await
            .map_err(anyhow::Error::from)?;
        DeleteResult::from_deleted_count(res.deleted_count)
    }
}

impl From<MongoIncidentRepo> for ArcIncidentRepo {
    fn from(value: MongoIncidentRepo) -> Self {
        Arc::new(value)
    }
}
