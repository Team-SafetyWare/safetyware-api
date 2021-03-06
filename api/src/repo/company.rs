use crate::db::coll;
use crate::repo::mongo_util::{FindStream, FromDeletedCount, FromMatchedCount};
use crate::repo::DeleteResult;
use crate::repo::{ItemStream, ReplaceResult};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[async_trait::async_trait]
pub trait CompanyRepo {
    async fn insert_one(&self, company: Company) -> anyhow::Result<()>;
    async fn replace_one(&self, company: Company) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Company>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

pub type DynCompanyRepo = dyn CompanyRepo + Send + Sync + 'static;

pub type ArcCompanyRepo = Arc<DynCompanyRepo>;

#[derive(Debug, Clone)]
pub struct MongoCompanyRepo {
    pub db: Database,
}

impl MongoCompanyRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Company> {
        self.db.collection(coll::COMPANY)
    }
}

#[async_trait::async_trait]
impl CompanyRepo for MongoCompanyRepo {
    async fn insert_one(&self, company: Company) -> anyhow::Result<()> {
        self.collection().insert_one(company, None).await?;
        Ok(())
    }

    async fn replace_one(&self, company: Company) -> ReplaceResult {
        let res = self
            .collection()
            .replace_one(bson::doc! {"_id": &company.id}, company, None)
            .await
            .map_err(anyhow::Error::from)?;
        ReplaceResult::from_matched_count(res.matched_count)
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Company>> {
        Ok(self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?)
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>> {
        self.collection().find_stream(None, None).await
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

impl From<MongoCompanyRepo> for ArcCompanyRepo {
    fn from(value: MongoCompanyRepo) -> Self {
        Arc::new(value)
    }
}
