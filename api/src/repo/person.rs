use crate::db::coll;
use crate::repo::{DeleteResult, ReplaceError, DeleteError};
use crate::repo::{ItemStream, ReplaceResult};
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use futures_util::TryStreamExt;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Person {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub company_id: String,
}

#[async_trait::async_trait]
pub trait PersonRepo
{
    async fn insert_one(&self, person: &Person) -> anyhow::Result<()>;
    async fn replace_one(&self, person: &Person) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Person>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Person>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
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
        self.db.collection(coll::PERSON)
    }
}

#[async_trait::async_trait]
impl PersonRepo for MongoPersonRepo {
    async fn insert_one(&self, person: &Person) -> anyhow::Result<()> {
        self.collection().insert_one(person, None).await?;
        Ok(())
    }

    async fn replace_one(&self, person: &Person) -> ReplaceResult {
        let id = &person.id;
        let query = bson::doc! {"_id": id};
        let res = self.collection()
            .replace_one(query, person, None)
            .await
            .map_err(anyhow::Error::from)?;
        match res.matched_count {
            0 => Err(ReplaceError::NotFound),
            _ => Ok(()),
        }
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Person>> {
        let filter = bson::doc! {"_id": id};
        let found = self.collection().find_one(filter, None).await?;
        Ok(found)
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Person>>> {
        let cursor = self.collection().find(None, None).await?;
        let stream = cursor.map_err(|e| e.into());
        Ok(Box::new(stream))
    }

    async fn delete_one(&self, id: &str) -> DeleteResult {
        let res = self.collection()
            .delete_one(bson::doc! {"_id": id}, None)
            .await
            .map_err(anyhow::Error::from)?;
        match res.deleted_count {
            0 => Err(DeleteError::NotFound),
            _ => Ok(()),
        }
    }
}
