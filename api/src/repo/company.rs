use crate::repo::mongo_common as mc;
use crate::repo::ItemStream;
use bson::oid::ObjectId;
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

#[derive(Debug, Clone)]
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
        mc::insert_one(company, self.collection()).await
    }

    async fn replace_one(&self, company: &Company) -> anyhow::Result<()> {
        mc::replace_one(company, self.collection(), |x| x.id).await
    }

    async fn find_one(&self, id: ObjectId) -> anyhow::Result<Option<Company>> {
        mc::find_one(id, self.collection()).await
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>> {
        mc::find(self.collection()).await
    }

    async fn delete_one(&self, id: ObjectId) -> anyhow::Result<()> {
        mc::delete_one(id, self.collection()).await
    }
}
