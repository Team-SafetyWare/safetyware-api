use crate::db::coll;
use crate::repo::mongo_util::FindStream;
use crate::repo::{DeleteError, DeleteResult, ReplaceError};
use crate::repo::{ItemStream, ReplaceResult};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[async_trait::async_trait]
pub trait CompanyRepo {
    async fn insert_one(&self, company: &Company) -> anyhow::Result<()>;
    async fn replace_one(&self, company: &Company) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Company>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

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
    async fn insert_one(&self, company: &Company) -> anyhow::Result<()> {
        self.collection().insert_one(company, None).await?;
        Ok(())
    }

    async fn replace_one(&self, company: &Company) -> ReplaceResult {
        let id = &company.id;
        let query = bson::doc! {"_id": id};
        let res = self
            .collection()
            .replace_one(query, company, None)
            .await
            .map_err(anyhow::Error::from)?;
        match res.matched_count {
            0 => Err(ReplaceError::NotFound),
            _ => Ok(()),
        }
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Company>> {
        let filter = bson::doc! {"_id": id};
        let found = self.collection().find_one(filter, None).await?;
        Ok(found)
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
        match res.deleted_count {
            0 => Err(DeleteError::NotFound),
            _ => Ok(()),
        }
    }
}
