use crate::db::coll;
use crate::repo::{DeleteError, DeleteResult, ReplaceError};
use crate::repo::{ItemStream, ReplaceResult};
use bson::Document;
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserAccount {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: String,
    pub company_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct UserAccountFilter {
    pub company_ids: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait UserAccountRepo {
    async fn insert_one(&self, user_account: &UserAccount) -> anyhow::Result<()>;
    async fn replace_one(&self, user_account: &UserAccount) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<UserAccount>>;
    async fn find(
        &self,
        filter: &UserAccountFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<UserAccount>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

#[derive(Debug, Clone)]
pub struct MongoUserAccountRepo {
    pub db: Database,
}

impl MongoUserAccountRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<UserAccount> {
        self.db.collection(coll::USER_ACCOUNT)
    }
}

#[async_trait::async_trait]
impl UserAccountRepo for MongoUserAccountRepo {
    async fn insert_one(&self, user_account: &UserAccount) -> anyhow::Result<()> {
        self.collection().insert_one(user_account, None).await?;
        Ok(())
    }

    async fn replace_one(&self, user_account: &UserAccount) -> ReplaceResult {
        let id = &user_account.id;
        let query = bson::doc! {"_id": id};
        let res = self
            .collection()
            .replace_one(query, user_account, None)
            .await
            .map_err(anyhow::Error::from)?;
        match res.matched_count {
            0 => Err(ReplaceError::NotFound),
            _ => Ok(()),
        }
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<UserAccount>> {
        let filter = bson::doc! {"_id": id};
        let found = self.collection().find_one(filter, None).await?;
        Ok(found)
    }

    async fn find(
        &self,
        filter: &UserAccountFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<UserAccount>>> {
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
}
