use crate::common::{GetId, HasId, SetId};
use crate::repo::op::{DeleteOne, Find, FindOne, InsertOne, ReplaceOne};
use crate::repo::{mongo_op, DeleteResult};
use crate::repo::{ItemStream, ReplaceResult};
use bson::oid::ObjectId;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

impl HasId for Company {
    type Id = ObjectId;
}

impl GetId for Company {
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl SetId for Company {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

#[async_trait::async_trait]
pub trait CompanyRepo:
    InsertOne<Company> + ReplaceOne<Company> + FindOne<Company> + Find<Company> + DeleteOne<Company>
{
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

impl CompanyRepo for MongoCompanyRepo {}

#[async_trait::async_trait]
impl InsertOne<Company> for MongoCompanyRepo {
    async fn insert_one(&self, item: &Company) -> anyhow::Result<()> {
        mongo_op::insert_one(item, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl ReplaceOne<Company> for MongoCompanyRepo {
    async fn replace_one(&self, item: &Company) -> ReplaceResult {
        mongo_op::replace_one(item, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl FindOne<Company> for MongoCompanyRepo {
    async fn find_one(&self, id: <Company as HasId>::Id) -> anyhow::Result<Option<Company>> {
        mongo_op::find_one(id, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl Find<Company> for MongoCompanyRepo {
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Company>>> {
        mongo_op::find(&self.collection()).await
    }
}

#[async_trait::async_trait]
impl DeleteOne<Company> for MongoCompanyRepo {
    async fn delete_one(&self, id: <Company as HasId>::Id) -> DeleteResult {
        mongo_op::delete_one(id, &self.collection()).await
    }
}
