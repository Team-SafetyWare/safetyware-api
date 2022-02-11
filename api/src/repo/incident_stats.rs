use crate::db::coll;
use crate::repo::ItemStream;
use bson::Document;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, TryStreamExt};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

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
        filter: &IncidentStatsFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<IncidentStats>>>;
}

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
        filter: &IncidentStatsFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<IncidentStats>>> {
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
        let cursor = self
            .collection()
            .aggregate(
                vec![
                    bson::doc! { "$match": mongo_filter },
                    bson::doc! { "$group": { "_id": "$type", "count": { "$sum": 1 } } },
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
