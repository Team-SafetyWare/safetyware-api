use crate::common::{GetId, HasId, SetId};
use crate::repo::op::{DeleteOne, Find, FindOne, InsertOne, ReplaceOne};
use crate::repo::{mongo_op, DeleteResult};
use crate::repo::{ItemStream, ReplaceResult};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Person {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub company_id: String,
}

impl HasId for Person {
    type Id = String;
}

impl GetId for Person {
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl SetId for Person {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

#[async_trait::async_trait]
pub trait PersonRepo:
    InsertOne<Person> + ReplaceOne<Person> + FindOne<Person> + Find<Person> + DeleteOne<Person>
{
}

#[derive(Debug, Clone)]
pub struct MongoPersonRepo {
    pub db: Database,
}

impl MongoPersonRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Person> {
        self.db.collection("company")
    }
}

impl PersonRepo for MongoPersonRepo {}

#[async_trait::async_trait]
impl InsertOne<Person> for MongoPersonRepo {
    async fn insert_one(&self, item: &Person) -> anyhow::Result<()> {
        mongo_op::insert_one(item, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl ReplaceOne<Person> for MongoPersonRepo {
    async fn replace_one(&self, item: &Person) -> ReplaceResult {
        mongo_op::replace_one(item, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl FindOne<Person> for MongoPersonRepo {
    async fn find_one(&self, id: &<Person as HasId>::Id) -> anyhow::Result<Option<Person>> {
        mongo_op::find_one(id, &self.collection()).await
    }
}

#[async_trait::async_trait]
impl Find<Person> for MongoPersonRepo {
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Person>>> {
        mongo_op::find(&self.collection()).await
    }
}

#[async_trait::async_trait]
impl DeleteOne<Person> for MongoPersonRepo {
    async fn delete_one(&self, id: &<Person as HasId>::Id) -> DeleteResult {
        mongo_op::delete_one(id, &self.collection()).await
    }
}
