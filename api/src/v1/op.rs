use crate::common::{HasId, NewId, SetId};
use crate::repo::op::{DeleteOne, Find, FindOne, InsertOne, ReplaceOne};
use crate::repo::{DeleteError, ReplaceError};
use crate::warp_ext;
use crate::warp_ext::AsJsonReply;
use crate::warp_ext::BoxReply;
use futures_util::TryStreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::str::FromStr;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Reply};

pub fn get<Item, RepoItem, Repo>(
    collection_name: String,
    repo: Arc<Repo>,
) -> BoxedFilter<(Box<dyn Reply>,)>
where
    Repo: FindOne<RepoItem> + Send + Sync + ?Sized + 'static,
    Item: HasId<Id = Option<String>> + From<RepoItem> + Serialize,
    RepoItem: HasId,
    RepoItem::Id: FromStr + Send + Sync,
    <RepoItem::Id as FromStr>::Err: Into<anyhow::Error>,
{
    warp::path(collection_name)
        .and(warp::get())
        .and(warp_ext::with_clone(repo))
        .and(warp::path::param())
        .then(move |repo: Arc<Repo>, id: String| async move {
            let rid: RepoItem::Id = id.parse().map_err(Into::into)?;
            let item = repo.find_one(&rid).await?.map(Item::from);
            let reply = match item {
                None => StatusCode::NOT_FOUND.boxed(),
                Some(item) => item.as_json_reply().boxed(),
            };
            Ok(reply)
        })
        .map(warp_ext::convert_err)
        .boxed()
}

pub fn list<Item, RepoItem, Repo>(
    collection_name: String,
    repo: Arc<Repo>,
) -> BoxedFilter<(Box<dyn Reply>,)>
where
    Repo: Find<RepoItem> + Send + Sync + ?Sized + 'static,
    Item: Serialize + Send,
    RepoItem: Into<Item>,
{
    warp::path(collection_name)
        .and(warp::get())
        .and(warp_ext::with_clone(repo))
        .then(move |repo: Arc<Repo>| async move {
            Ok(repo
                .find()
                .await?
                .map_ok(Into::into)
                .try_collect::<Vec<Item>>()
                .await?
                .as_json_reply()
                .boxed())
        })
        .map(warp_ext::convert_err)
        .boxed()
}

pub fn create<Item, RepoItem, Repo>(
    collection_name: String,
    repo: Arc<Repo>,
) -> BoxedFilter<(Box<dyn Reply>,)>
where
    Repo: InsertOne<RepoItem> + Send + Sync + ?Sized + 'static,
    Item: HasId<Id = Option<String>>
        + NewId
        + SetId
        + TryInto<RepoItem>
        + Clone
        + DeserializeOwned
        + Serialize
        + Send
        + 'static,
    <Item as TryInto<RepoItem>>::Error: Into<anyhow::Error>,
    RepoItem: Send,
{
    warp::path(collection_name)
        .and(warp::post())
        .and(warp_ext::with_clone(repo))
        .and(warp::body::json())
        .then(move |repo: Arc<Repo>, mut item: Item| async move {
            item.set_id(Item::new_id());
            let repo_item = item.clone().try_into().map_err(Into::into)?;
            let fut = repo.insert_one(&repo_item);
            fut.await?;
            let reply = item.as_json_reply().boxed();
            Ok(reply)
        })
        .map(warp_ext::convert_err)
        .boxed()
}

pub fn delete<RepoItem, Repo>(
    collection_name: String,
    repo: Arc<Repo>,
) -> BoxedFilter<(Box<dyn Reply>,)>
where
    Repo: DeleteOne<RepoItem> + Send + Sync + ?Sized + 'static,
    RepoItem: HasId,
    RepoItem::Id: FromStr + Send,
    <RepoItem::Id as FromStr>::Err: Into<anyhow::Error>,
{
    warp::path(collection_name)
        .and(warp::delete())
        .and(warp_ext::with_clone(repo))
        .and(warp::path::param())
        .then(move |repo: Arc<Repo>, id: String| async move {
            let rid: RepoItem::Id = id.parse().map_err(Into::into)?;
            let fut = repo.delete_one(&rid);
            let res = fut.await;
            match res {
                Ok(()) => Ok(warp::reply().boxed()),
                Err(DeleteError::NotFound) => Ok(StatusCode::NOT_FOUND.boxed()),
                Err(e) => Err(e.into()),
            }
        })
        .map(warp_ext::convert_err)
        .boxed()
}

pub fn replace<Item, RepoItem, Repo>(
    collection_name: String,
    repo: Arc<Repo>,
) -> BoxedFilter<(Box<dyn Reply>,)>
where
    Repo: ReplaceOne<RepoItem> + Send + Sync + ?Sized + 'static,
    Item: HasId<Id = Option<String>>
        + SetId
        + Clone
        + TryInto<RepoItem>
        + Serialize
        + DeserializeOwned
        + Send
        + 'static,
    <Item as TryInto<RepoItem>>::Error: Into<anyhow::Error>,
    RepoItem: Send,
{
    warp::path(collection_name)
        .and(warp::put())
        .and(warp_ext::with_clone(repo))
        .and(warp::path::param())
        .and(warp::body::json())
        .then(
            move |repo: Arc<Repo>, id: String, mut item: Item| async move {
                item.set_id(Some(id.parse()?));
                let repo_item = item.clone().try_into().map_err(Into::into)?;
                let fut = repo.replace_one(&repo_item);
                let res = fut.await;
                match res {
                    Ok(()) => Ok(item.as_json_reply().boxed()),
                    Err(ReplaceError::NotFound) => Ok(StatusCode::NOT_FOUND.boxed()),
                    Err(e) => Err(e.into()),
                }
            },
        )
        .map(warp_ext::convert_err)
        .boxed()
}

//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::GetId;
    use crate::crockford;
    use crate::repo::{mem_op, DeleteResult, ItemStream, ReplaceResult};
    use anyhow::Context;
    use serde::Deserialize;
    use std::convert::TryFrom;

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct RepoItem {
        pub id: String,
        pub name: String,
    }

    impl HasId for RepoItem {
        type Id = String;
    }

    impl GetId for RepoItem {
        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    impl SetId for RepoItem {
        fn set_id(&mut self, id: Self::Id) {
            self.id = id
        }
    }

    #[async_trait::async_trait]
    pub trait ItemRepo:
        InsertOne<RepoItem>
        + ReplaceOne<RepoItem>
        + FindOne<RepoItem>
        + Find<RepoItem>
        + DeleteOne<RepoItem>
    {
    }

    #[derive(Debug, Clone)]
    pub struct MemoryItemRepo {
        pub collection: mem_op::Collection<RepoItem>,
    }

    impl Default for MemoryItemRepo {
        fn default() -> Self {
            Self {
                collection: Default::default(),
            }
        }
    }

    impl ItemRepo for MemoryItemRepo {}

    #[async_trait::async_trait]
    impl InsertOne<RepoItem> for MemoryItemRepo {
        async fn insert_one(&self, item: &RepoItem) -> anyhow::Result<()> {
            mem_op::insert_one(item, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl ReplaceOne<RepoItem> for MemoryItemRepo {
        async fn replace_one(&self, item: &RepoItem) -> ReplaceResult {
            mem_op::replace_one(item, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl FindOne<RepoItem> for MemoryItemRepo {
        async fn find_one(&self, id: &<RepoItem as HasId>::Id) -> anyhow::Result<Option<RepoItem>> {
            mem_op::find_one(id, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl Find<RepoItem> for MemoryItemRepo {
        async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<RepoItem>>> {
            mem_op::find(&self.collection)
        }
    }

    #[async_trait::async_trait]
    impl DeleteOne<RepoItem> for MemoryItemRepo {
        async fn delete_one(&self, id: &<RepoItem as HasId>::Id) -> DeleteResult {
            mem_op::delete_one(id, &self.collection)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiItem {
        #[serde(skip_deserializing)]
        pub id: Option<String>,
        pub name: String,
    }

    impl From<RepoItem> for ApiItem {
        fn from(value: RepoItem) -> Self {
            Self {
                id: Some(value.id.to_string()),
                name: value.name,
            }
        }
    }

    impl TryFrom<ApiItem> for RepoItem {
        type Error = anyhow::Error;

        fn try_from(value: ApiItem) -> Result<Self, Self::Error> {
            Ok(Self {
                id: value.id.context("id missing")?.parse()?,
                name: value.name,
            })
        }
    }

    impl HasId for ApiItem {
        type Id = Option<String>;
    }

    impl GetId for ApiItem {
        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    impl SetId for ApiItem {
        fn set_id(&mut self, id: Self::Id) {
            self.id = id
        }
    }

    impl NewId for ApiItem {
        fn new_id() -> Self::Id {
            Some(crockford::random_id())
        }
    }
}
