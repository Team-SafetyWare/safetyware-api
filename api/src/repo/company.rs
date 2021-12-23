use crate::repo::ItemStream;
use bson::oid::ObjectId;
use futures_util::stream::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

#[async_trait::async_trait]
pub trait CompanyRepo {
    async fn insert_one(&self, company: &Company) -> anyhow::Result<()>;
    async fn replace_one(&self, company: &Company) -> anyhow::Result<()>;
    async fn find_one(&self, id: ObjectId) -> anyhow::Result<Option<Company>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>>;
    async fn delete_one(&self, id: ObjectId) -> anyhow::Result<()>;
}

pub struct MongoCompanyRepo {
    pub db: Database,
}

impl MongoCompanyRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Company> {
        self.db.collection("company")
    }
}

#[async_trait::async_trait]
impl CompanyRepo for MongoCompanyRepo {
    async fn insert_one(&self, company: &Company) -> anyhow::Result<()> {
        self.collection().insert_one(company, None).await?;
        Ok(())
    }

    async fn replace_one(&self, company: &Company) -> anyhow::Result<()> {
        self.collection()
            .replace_one(bson::doc! {"_id": company.id}, company, None)
            .await?;
        Ok(())
    }

    async fn find_one(&self, id: ObjectId) -> anyhow::Result<Option<Company>> {
        let found = self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?;
        Ok(found)
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>> {
        let cursor = self.collection().find(None, None).await?;
        let stream = cursor.map_err(|e| e.into());
        Ok(Box::new(stream))
    }

    async fn delete_one(&self, id: ObjectId) -> anyhow::Result<()> {
        self.collection()
            .delete_one(bson::doc! {"_id": id}, None)
            .await?;
        Ok(())
    }
}
