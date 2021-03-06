use crate::db::coll;
use crate::repo::mongo_util::{filter, FindStream, FromDeletedCount, FromMatchedCount, InsertOpt};
use crate::repo::DeleteResult;
use crate::repo::{ItemStream, ReplaceResult};
use bson::Document;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Device {
    #[serde(rename = "_id")]
    pub id: String,
    pub owner_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct DeviceFilter {
    pub owner_ids: Option<Vec<String>>,
}

#[async_trait::async_trait]
pub trait DeviceRepo {
    async fn insert_one(&self, device: Device) -> anyhow::Result<()>;
    async fn replace_one(&self, device: Device) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Device>>;
    async fn find(&self, filter: DeviceFilter) -> anyhow::Result<Box<dyn ItemStream<Device>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

pub type DynDeviceRepo = dyn DeviceRepo + Send + Sync + 'static;

pub type ArcDeviceRepo = Arc<DynDeviceRepo>;

#[derive(Debug, Clone)]
pub struct MongoDeviceRepo {
    pub db: Database,
}

impl MongoDeviceRepo {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn collection(&self) -> Collection<Device> {
        self.db.collection(coll::DEVICE)
    }
}

#[async_trait::async_trait]
impl DeviceRepo for MongoDeviceRepo {
    async fn insert_one(&self, device: Device) -> anyhow::Result<()> {
        self.collection().insert_one(device, None).await?;
        Ok(())
    }

    async fn replace_one(&self, device: Device) -> ReplaceResult {
        let res = self
            .collection()
            .replace_one(bson::doc! {"_id": &device.id}, device, None)
            .await
            .map_err(anyhow::Error::from)?;
        ReplaceResult::from_matched_count(res.matched_count)
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Device>> {
        Ok(self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?)
    }

    async fn find(&self, filter: DeviceFilter) -> anyhow::Result<Box<dyn ItemStream<Device>>> {
        let mut mongo_filter = Document::new();
        mongo_filter.insert_opt("owner_id", filter::one_of(filter.owner_ids));
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

impl From<MongoDeviceRepo> for ArcDeviceRepo {
    fn from(value: MongoDeviceRepo) -> Self {
        Arc::new(value)
    }
}
