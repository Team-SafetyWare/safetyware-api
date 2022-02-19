use crate::repo::ItemStream;
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
        match values {
            None => None,
            Some(values) => Some((bson::doc! { "$in":  values }).into()),
        }
    }
}
