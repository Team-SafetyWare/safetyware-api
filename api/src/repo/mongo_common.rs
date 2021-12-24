use crate::repo::ItemStream;
use bson::Bson;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait Item: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static {}

impl<T> Item for T where T: Serialize + DeserializeOwned + Unpin + Send + Sync + 'static {}

pub async fn insert_one<T>(item: &T, collection: Collection<T>) -> anyhow::Result<()>
where
    T: Item,
{
    collection.insert_one(item, None).await?;
    Ok(())
}

pub async fn replace_one<T, K, F>(
    item: &T,
    collection: Collection<T>,
    get_id: F,
) -> anyhow::Result<()>
where
    T: Item,
    Bson: From<K>,
    F: Fn(&T) -> K,
{
    let id = get_id(item);
    let query = bson::doc! {"_id": id};
    collection.replace_one(query, item, None).await?;
    Ok(())
}

pub async fn find_one<T, K>(id: K, collection: Collection<T>) -> anyhow::Result<Option<T>>
where
    T: Item,
    Bson: From<K>,
{
    let filter = bson::doc! {"_id": id};
    let found = collection.find_one(filter, None).await?;
    Ok(found)
}

pub async fn find<T>(collection: Collection<T>) -> anyhow::Result<Box<dyn ItemStream<T>>>
where
    T: Item,
{
    let cursor = collection.find(None, None).await?;
    let stream = cursor.map_err(|e| e.into());
    Ok(Box::new(stream))
}

pub async fn delete_one<T, K>(id: K, collection: Collection<T>) -> anyhow::Result<()>
where
    T: Item,
    Bson: From<K>,
{
    collection.delete_one(bson::doc! {"_id": id}, None).await?;
    Ok(())
}
