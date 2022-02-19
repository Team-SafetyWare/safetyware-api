use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, InsertOpt};
use crate::repo::{DeleteError, DeleteResult, ReplaceError};
use crate::repo::{ItemStream, ReplaceResult};
use bson::Document;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Person {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub company_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct PersonFilter {
    pub company_ids: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait PersonRepo {
    async fn insert_one(&self, person: &Person) -> anyhow::Result<()>;
    async fn replace_one(&self, person: &Person) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Person>>;
    async fn find(&self, filter: PersonFilter) -> anyhow::Result<Box<dyn ItemStream<Person>>>;
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
        let res = self
            .collection()
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

    async fn find(&self, filter: PersonFilter) -> anyhow::Result<Box<dyn ItemStream<Person>>> {
        let mut mongo_filter = Document::new();
        mongo_filter.insert_opt("company_id", filter::one_of(filter.company_ids));
        self.collection().find_stream(mongo_filter, None).await
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
