use crate::db::coll;
use crate::repo::ItemStream;
use crate::repo::{DeleteError, DeleteResult};
use bson::Document;
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

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TeamPerson {
    #[serde(rename = "_id")]
    pub team_id: String,
    pub person_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct TeamFilter {
    pub company_ids: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait TeamRepo {
    async fn insert_one(&self, team: &Team) -> anyhow::Result<()>;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Team>>;
    async fn find(&self, filter: &TeamFilter) -> anyhow::Result<Box<dyn ItemStream<Team>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
    async fn find_people(&self, team_id: &str) -> anyhow::Result<Box<dyn ItemStream<TeamPerson>>>;
    async fn add_person(&self, team_id: &str, person_id: &str) -> anyhow::Result<()>;
    async fn remove_person(&self, team_id: &str, person_id: &str) -> DeleteResult;
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

    pub fn person_collection(&self) -> Collection<TeamPerson> {
        self.db.collection(coll::TEAM_PERSON)
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

    async fn find(&self, filter: &TeamFilter) -> anyhow::Result<Box<dyn ItemStream<Team>>> {
        let mut mongo_filter = Document::new();
        if let Some(company_ids) = &filter.company_ids {
            mongo_filter.insert("company_id", bson::doc! { "$in": company_ids });
        }
        let cursor = self.collection().find(mongo_filter, None).await?;
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

    async fn find_people(&self, team_id: &str) -> anyhow::Result<Box<dyn ItemStream<TeamPerson>>> {
        let mongo_filter = bson::doc! { "team_id": team_id };
        let cursor = self.person_collection().find(mongo_filter, None).await?;
        let stream = cursor.map_err(|e| e.into());
        Ok(Box::new(stream))
    }

    async fn add_person(&self, team_id: &str, person_id: &str) -> anyhow::Result<()> {
        self.person_collection()
            .insert_one(
                TeamPerson {
                    team_id: team_id.to_string(),
                    person_id: person_id.to_string(),
                },
                None,
            )
            .await?;
        Ok(())
    }

    async fn remove_person(&self, team_id: &str, person_id: &str) -> DeleteResult {
        let res = self
            .person_collection()
            .delete_one(
                bson::doc! { "team_id": team_id, "person_id": person_id, },
                None,
            )
            .await
            .map_err(anyhow::Error::from)?;
        match res.deleted_count {
            0 => Err(DeleteError::NotFound),
            _ => Ok(()),
        }
    }
}
