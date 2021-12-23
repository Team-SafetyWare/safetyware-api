use crate::repo::{Item, MongoRepo, Repo};
use bson::oid::ObjectId;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

impl Item for Company {
    type Key = ObjectId;

    fn id(&self) -> Self::Key {
        self.id
    }
}

#[async_trait::async_trait]
pub trait CompanyRepo: Repo<Company> {}

pub struct MongoCompanyRepo {
    pub db: Database,
}

impl MongoCompanyRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl MongoRepo<Company> for MongoCompanyRepo {
    fn collection(&self) -> Collection<Company> {
        self.db.collection("company")
    }
}

#[async_trait::async_trait]
impl CompanyRepo for MongoCompanyRepo {}
