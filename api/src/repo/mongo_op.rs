use crate::common::{GetId, HasId};
use crate::repo::{DeleteError, DeleteResult, ItemStream, ReplaceError, ReplaceResult};
use bson::Bson;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Item: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static {}

impl<T> Item for T where T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static {}

pub async fn insert_one<T>(item: &T, collection: &Collection<T>) -> anyhow::Result<()>
where
    T: Item,
{
    collection.insert_one(item, None).await?;
    Ok(())
}

pub async fn replace_one<T>(item: &T, collection: &Collection<T>) -> ReplaceResult
where
    T: Item + GetId,
    Bson: From<T::Id>,
{
    let id = item.id();
    let query = bson::doc! {"_id": id};
    let res = collection
        .replace_one(query, item, None)
        .await
        .map_err(anyhow::Error::from)?;
    match res.matched_count {
        0 => Err(ReplaceError::NotFound),
        _ => Ok(()),
    }
}

pub async fn find_one<T>(id: T::Id, collection: &Collection<T>) -> anyhow::Result<Option<T>>
where
    T: Item + HasId,
    Bson: From<T::Id>,
{
    let filter = bson::doc! {"_id": id};
    let found = collection.find_one(filter, None).await?;
    Ok(found)
}

pub async fn find<T>(collection: &Collection<T>) -> anyhow::Result<Box<dyn ItemStream<T>>>
where
    T: Item,
{
    let cursor = collection.find(None, None).await?;
    let stream = cursor.map_err(|e| e.into());
    Ok(Box::new(stream))
}

pub async fn delete_one<T>(id: T::Id, collection: &Collection<T>) -> DeleteResult
where
    T: Item + HasId,
    Bson: From<T::Id>,
{
    let res = collection
        .delete_one(bson::doc! {"_id": id}, None)
        .await
        .map_err(anyhow::Error::from)?;
    match res.deleted_count {
        0 => Err(DeleteError::NotFound),
        _ => Ok(()),
    }
}
