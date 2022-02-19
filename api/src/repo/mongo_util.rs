use crate::repo::{DeleteError, DeleteResult, ItemStream, ReplaceError, ReplaceResult};
use bson::{Bson, Document};
use futures_util::TryStreamExt;
use mongodb::options::FindOptions;
use mongodb::Collection;
use serde::de::DeserializeOwned;

pub trait InsertOpt {
    fn insert_opt<KT: Into<String>, BT: Into<Bson>>(
        &mut self,
        key: KT,
        val: Option<BT>,
    ) -> Option<Bson>;
}

impl InsertOpt for Document {
    fn insert_opt<KT: Into<String>, BT: Into<Bson>>(
        &mut self,
        key: KT,
        val: Option<BT>,
    ) -> Option<Bson> {
        match val {
            None => None,
            Some(val) => self.insert(key, val),
        }
    }
}

#[async_trait::async_trait]
pub trait FindStream {
    type Item;

    async fn find_stream<T>(
        &self,
        filter: impl Into<Option<Document>> + Send + 'async_trait,
        options: impl Into<Option<FindOptions>> + Send + 'async_trait,
    ) -> anyhow::Result<Box<dyn ItemStream<T>>>
    where
        T: Unpin + Send + 'static,
        Self::Item: Into<T>;
}

#[async_trait::async_trait]
impl<D: DeserializeOwned + Unpin + Send + Sync + 'static> FindStream for Collection<D> {
    type Item = D;

    async fn find_stream<T>(
        &self,
        filter: impl Into<Option<Document>> + Send + 'async_trait,
        options: impl Into<Option<FindOptions>> + Send + 'async_trait,
    ) -> anyhow::Result<Box<dyn ItemStream<T>>>
    where
        T: Unpin + Send + 'static,
        Self::Item: Into<T>,
    {
        let cursor = self.find(filter, options).await?;
        let stream = cursor.map_ok(Into::into).map_err(|e| e.into());
        Ok(Box::new(stream))
    }
}

pub trait FromMatchedCount {
    fn from_matched_count(matched_count: u64) -> Self;
}

impl FromMatchedCount for ReplaceResult {
    fn from_matched_count(matched_count: u64) -> Self {
        match matched_count {
            0 => Err(ReplaceError::NotFound),
            _ => Ok(()),
        }
    }
}

pub trait FromDeletedCount {
    fn from_deleted_count(deleted_count: u64) -> Self;
}

impl FromDeletedCount for DeleteResult {
    fn from_deleted_count(deleted_count: u64) -> Self {
        match deleted_count {
            0 => Err(DeleteError::NotFound),
            _ => Ok(()),
        }
    }
}

pub mod filter {
    use bson::{Bson, Document};

    pub fn clamp<T: Into<Bson>>(
        min_inclusive: Option<T>,
        max_exclusive: Option<T>,
    ) -> Option<Bson> {
        let mut doc: Option<Document> = None;
        if let Some(min) = min_inclusive {
            doc.get_or_insert_with(Default::default).insert("$gte", min);
        }
        if let Some(max) = max_exclusive {
            doc.get_or_insert_with(Default::default).insert("$lt", max);
        }
        doc.map(Into::into)
    }

    pub fn one_of<T: Into<Bson>>(values: Option<Vec<T>>) -> Option<Bson> {
        values.map(|v| (bson::doc! { "$in":  v }).into())
    }
}
