use crate::repo::company::Company;
use crate::repo::{DeleteResult, ItemStream};

pub trait HasId {
    type Id;
}

pub trait GetId: HasId {
    fn id(&self) -> Self::Id;
}

pub trait SetId: HasId {
    fn set_id(&mut self, id: Self::Id);
}

#[async_trait::async_trait]
pub trait InsertOne<T> {
    async fn insert_one(&self, item: &T) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait ReplaceOne<T> {
    async fn replace_one(&self, item: &T) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait FindOne<T: HasId> {
    async fn find_one(&self, id: T::Id) -> anyhow::Result<Option<T>>;
}

#[async_trait::async_trait]
pub trait Find<T> {
    async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<T>>>;
}

#[async_trait::async_trait]
pub trait DeleteOne<T: HasId> {
    async fn delete_one(&self, id: T::Id) -> anyhow::Result<DeleteResult>;
}
