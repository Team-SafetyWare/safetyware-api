use crate::db::coll;
use crate::repo::{DeleteError, DeleteResult, ReplaceError};
use crate::repo::{ItemStream, ReplaceResult};
use bson::Document;
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

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
    async fn insert_one(&self, device: &Device) -> anyhow::Result<()>;
    async fn replace_one(&self, device: &Device) -> ReplaceResult;
    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Device>>;
    async fn find(&self, filter: DeviceFilter) -> anyhow::Result<Box<dyn ItemStream<Device>>>;
    async fn delete_one(&self, id: &str) -> DeleteResult;
}

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
    async fn insert_one(&self, device: &Device) -> anyhow::Result<()> {
        self.collection().insert_one(device, None).await?;
        Ok(())
    }

    async fn replace_one(&self, device: &Device) -> ReplaceResult {
        let id = &device.id;
        let query = bson::doc! {"_id": id};
        let res = self
            .collection()
            .replace_one(query, device, None)
            .await
            .map_err(anyhow::Error::from)?;
        match res.matched_count {
            0 => Err(ReplaceError::NotFound),
            _ => Ok(()),
        }
    }

    async fn find_one(&self, id: &str) -> anyhow::Result<Option<Device>> {
        let filter = bson::doc! {"_id": id};
        let found = self.collection().find_one(filter, None).await?;
        Ok(found)
    }

    async fn find(&self, filter: DeviceFilter) -> anyhow::Result<Box<dyn ItemStream<Device>>> {
        let mut mongo_filter = Document::new();
        if let Some(owner_ids) = filter.owner_ids {
            mongo_filter.insert("owner_id", bson::doc! { "$in": owner_ids });
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
