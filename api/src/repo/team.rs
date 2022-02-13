use crate::db::coll;
use crate::repo::{DeleteError, DeleteResult};
use crate::repo::{ItemStream};
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Team {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub company_id: String,
}

#[async_trait::async_trait]
pub trait TeamRepo {
    async fn insert_one(&self, team: &Team) -> anyhow::Result<()>;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Team>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Team>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

#[derive(Debug, Clone)]
pub struct MongoTeamRepo {
    pub db: Database,
}

impl MongoTeamRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Team> {
        self.db.collection(coll::TEAM)
    }
}

#[async_trait::async_trait]
impl TeamRepo for MongoTeamRepo {
    async fn insert_one(&self, team: &Team) -> anyhow::Result<()> {
        self.collection().insert_one(team, None).await?;
        Ok(())
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Team>> {
        let filter = bson::doc! {"_id": id};
        let found = self.collection().find_one(filter, None).await?;
        Ok(found)
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<Team>>> {
        let cursor = self.collection().find(None, None).await?;
        let stream = cursor.map_err(|e| e.into());
        Ok(Box::new(stream))
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
