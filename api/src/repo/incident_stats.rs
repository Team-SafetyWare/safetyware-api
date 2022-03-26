use crate::db::coll;
use crate::repo::mongo_util::{filter, InsertOpt};
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, TryStreamExt};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentStats {
    pub r#type: String,
    pub count: i32,
}

#[derive(Default, Debug, Clone)]
pub struct IncidentStatsFilter {
    pub person_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
pub trait IncidentStatsRepo {
    async fn find(
        &self,
        filter: IncidentStatsFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<IncidentStats>>>;
}

pub type DynIncidentStatsRepo = dyn IncidentStatsRepo + Send + Sync + 'static;

pub type ArcIncidentStatsRepo = Arc<DynIncidentStatsRepo>;

#[derive(Debug, Clone)]
pub struct MongoIncidentStatsRepo {
    pub db: Database,
}

impl MongoIncidentStatsRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Document> {
        self.db.collection(coll::INCIDENT)
    }
}

#[async_trait::async_trait]
impl IncidentStatsRepo for MongoIncidentStatsRepo {
    async fn find(
        &self,
        filter: IncidentStatsFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<IncidentStats>>> {
        let mut mongo_filter = Document::new();
        mongo_query.insert("hidden", filter::not_true());
        mongo_filter.insert_opt("person_id", filter::one_of(filter.person_ids));
        mongo_filter.insert_opt(
            "timestamp",
            filter::clamp(filter.min_timestamp, filter.max_timestamp),
        );
        let cursor = self
            .collection()
            .aggregate(
                vec![
                    bson::doc! { "$match": mongo_filter },
                    bson::doc! { "$group": { "_id": "$type", "count": { "$sum": 1 } } },
                    bson::doc! { "$set": { "type": "$_id" } },
                ],
                None,
            )
            .await?;
        let stream = cursor
            .map_err(anyhow::Error::from)
            .map(|r| r.and_then(|d| bson::from_document(d).map_err(Into::into)));
        Ok(Box::new(stream))
    }
}

impl From<MongoIncidentStatsRepo> for ArcIncidentStatsRepo {
    fn from(value: MongoIncidentStatsRepo) -> Self {
        Arc::new(value)
    }
}
