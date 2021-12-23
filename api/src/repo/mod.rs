use bson::Bson;
use futures_util::Stream;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod company;

pub trait ItemStream<T>: Stream<Item = anyhow::Result<T>> {}

impl<T, I> ItemStream<I> for T where T: Stream<Item = anyhow::Result<I>> {}

pub trait Item {
    type Key;

    fn id(&self) -> Self::Key;
}

#[async_trait::async_trait]
pub trait Repo<T: Item> {
    async fn insert_one(&self, item: &T) -> anyhow::Result<()>;
    async fn replace_one(&self, item: &T) -> anyhow::Result<()>;
    async fn find_one(&self, id: T::Key) -> anyhow::Result<Option<T>>;
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<T>>>;
    async fn delete_one(&self, id: T::Key) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait MongoRepo<T>: Repo<T>
where
    T: Item + Serialize + DeserializeOwned + Unpin + Send + Sync + 'static,
    T::Key: Send,
    Bson: From<<T as Item>::Key>,
{
    fn collection(&self) -> Collection<T>;

    async fn insert_one(&self, item: &T) -> anyhow::Result<()> {
        self.collection().insert_one(item, None).await?;
        Ok(())
    }

    async fn replace_one(&self, item: &T) -> anyhow::Result<()> {
        self.collection()
            .replace_one(bson::doc! {"_id": item.id()}, item, None)
            .await?;
        Ok(())
    }

    async fn find_one(&self, id: T::Key) -> anyhow::Result<Option<T>> {
        let found = self
            .collection()
            .find_one(bson::doc! {"_id": id}, None)
            .await?;
        Ok(found)
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<T>>> {
        let cursor = self.collection().find(None, None).await?;
        let stream = cursor.map_err(|e| e.into());
        Ok(Box::new(stream))
    }

    async fn delete_one(&self, id: T::Key) -> anyhow::Result<()> {
        self.collection()
            .delete_one(bson::doc! {"_id": id}, None)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T, I> Repo<I> for T
where
    I: Item + Serialize + DeserializeOwned + Unpin + Send + Sync + 'static,
    I::Key: Send,
    Bson: From<<I as Item>::Key>,
    T: MongoRepo<I> + Sync,
{
    async fn insert_one(&self, item: &I) -> anyhow::Result<()> {
        MongoRepo::insert_one(self, item).await
    }

    async fn replace_one(&self, item: &I) -> anyhow::Result<()> {
        MongoRepo::replace_one(self, item).await
    }

    async fn find_one(&self, id: I::Key) -> anyhow::Result<Option<I>> {
        MongoRepo::find_one(self, id).await
    }

    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<I>>> {
        MongoRepo::find(self).await
    }

    async fn delete_one(&self, id: I::Key) -> anyhow::Result<()> {
        MongoRepo::delete_one(self, id).await
    }
}
