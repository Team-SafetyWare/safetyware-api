use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, FromDeletedCount, FromMatchedCount, InsertOpt};
use crate::repo::DeleteResult;
use crate::repo::{ItemStream, ReplaceResult};
use bson::Document;
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
    async fn insert_one(&self, user_account: UserAccount) -> anyhow::Result<()>;
    async fn replace_one(&self, user_account: UserAccount) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<UserAccount>>;
    async fn find(
        &self,
        filter: UserAccountFilter,
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
    async fn insert_one(&self, user_account: UserAccount) -> anyhow::Result<()> {
        self.collection().insert_one(user_account, None).await?;
        Ok(())
    }

    async fn replace_one(&self, user_account: UserAccount) -> ReplaceResult {
        let res = self
            .collection()
            .replace_one(bson::doc! {"_id": &user_account.id}, user_account, None)
            .await
            .map_err(anyhow::Error::from)?;
        ReplaceResult::from_matched_count(res.matched_count)
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<UserAccount>> {
        Ok(self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?)
    }

    async fn find(
        &self,
        filter: UserAccountFilter,
    ) -> anyhow::Result<Box<dyn ItemStream<UserAccount>>> {
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
        DeleteResult::from_deleted_count(res.deleted_count)
    }
}
