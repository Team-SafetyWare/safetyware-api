use crate::db::coll;
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

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
    async fn insert_many(&self, incidents: &[Incident]) -> anyhow::Result<()>;

    async fn find(&self, filter: &IncidentFilter) -> anyhow::Result<Box<dyn ItemStream<Incident>>>;
}

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
    async fn insert_many(&self, incidents: &[Incident]) -> anyhow::Result<()> {
        let db_incidents: Vec<DbIncident> =
            incidents.to_vec().into_iter().map(|r| r.into()).collect();
        self.collection().insert_many(db_incidents, None).await?;
        Ok(())
    }

    async fn find(&self, filter: &IncidentFilter) -> anyhow::Result<Box<dyn ItemStream<Incident>>> {
        let mut mongo_filter = Document::new();
        if let Some(person_ids) = &filter.person_ids {
            mongo_filter.insert("person_id", bson::doc! { "$in": person_ids });
        }
        if let Some(min_timestamp) = &filter.min_timestamp {
            mongo_filter
                .entry("timestamp".to_string())
                .or_insert(bson::doc! {}.into())
                .as_document_mut()
                .unwrap()
                .insert("$gte", min_timestamp);
        }
        if let Some(max_timestamp) = &filter.max_timestamp {
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
