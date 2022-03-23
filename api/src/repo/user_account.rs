use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, FromDeletedCount, FromMatchedCount, InsertOpt};
use crate::repo::DeleteResult;
use crate::repo::{ItemStream, ReplaceResult};
use bson::spec::BinarySubtype;
use bson::Document;
use mongodb::options::UpdateOptions;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub struct Creds {
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct DbCreds {
    pub user_account_id: String,
    pub password_hash: String,
}

impl From<DbCreds> for Creds {
    fn from(value: DbCreds) -> Self {
        Self {
            password_hash: value.password_hash,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileImage {
    pub user_account_id: String,
    pub image_png: bson::Binary,
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
    async fn credentials(&self, user_account_id: &str) -> anyhow::Result<Option<Creds>>;
    async fn set_creds(&self, user_account_id: &str, creds: Creds) -> anyhow::Result<()>;
    async fn profile_image_png(&self, user_account_id: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn set_profile_image_png(
        &self,
        user_account_id: &str,
        png_bytes: Vec<u8>,
    ) -> anyhow::Result<()>;
}

pub type DynUserAccountRepo = dyn UserAccountRepo + Send + Sync + 'static;

pub type ArcUserAccountRepo = Arc<DynUserAccountRepo>;

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

    pub fn creds_collection(&self) -> Collection<DbCreds> {
        self.db.collection(coll::USER_ACCOUNT_CREDS)
    }

    pub fn profile_image_collection(&self) -> Collection<ProfileImage> {
        self.db.collection(coll::USER_ACCOUNT_PROFILE_IMAGE)
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

    async fn credentials(&self, user_account_id: &str) -> anyhow::Result<Option<Creds>> {
        unimplemented!()
    }

    async fn set_creds(&self, user_account_id: &str, creds: Creds) -> anyhow::Result<()> {
        unimplemented!()
    }

    async fn profile_image_png(&self, user_account_id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let opt = self
            .profile_image_collection()
            .find_one(bson::doc! {"user_account_id": user_account_id}, None)
            .await?;
        match opt {
            None => Ok(None),
            Some(doc) => Ok(Some(doc.image_png.bytes)),
        }
    }

    async fn set_profile_image_png(
        &self,
        user_account_id: &str,
        png_bytes: Vec<u8>,
    ) -> anyhow::Result<()> {
        self.profile_image_collection()
            .update_one(
                bson::doc! {"user_account_id": user_account_id},
                bson::doc! { "$set" : bson::to_document(&ProfileImage {
                    user_account_id: user_account_id.to_string(),
                    image_png: bson::Binary {
                        subtype: BinarySubtype::Generic,
                        bytes: png_bytes,
                    },
                })? },
                UpdateOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }
}

impl From<MongoUserAccountRepo> for ArcUserAccountRepo {
    fn from(value: MongoUserAccountRepo) -> Self {
        Arc::new(value)
    }
}
