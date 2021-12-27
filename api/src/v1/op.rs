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
    use crate::v1::ResourceApi;
    use anyhow::Context;
    use serde::Deserialize;
    use std::convert::TryFrom;
    use std::future::Future;

    #[tokio::test]
    async fn test_create() {
        test_filter(|filter| async move {
            // Arrange.
            let item = ApiItem {
                id: None,
                name: crockford::random_id(),
            };

            // Act.
            let res = warp::test::request()
                .method("POST")
                .path("/items")
                .body(serde_json::to_string(&item).unwrap())
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::OK);
            let created: ApiItem = serde_json::from_slice(res.body()).unwrap();
            assert!(!created.id.unwrap().is_empty());
            assert_eq!(created.name, item.name);
        })
        .await
    }

    #[tokio::test]
    async fn test_create_invalid() {
        test_filter(|filter| async move {
            // Act.
            let res = warp::test::request()
                .method("POST")
                .path("/items")
                .body(r#"{"nonexistent":"nonexistent"}"#)
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_existing() {
        test_filter(|filter| async move {
            // Arrange.
            let item = ApiItem {
                id: None,
                name: crockford::random_id(),
            };
            let create_res = warp::test::request()
                .method("POST")
                .path("/items")
                .body(serde_json::to_string(&item).unwrap())
                .reply(&filter)
                .await;
            let created: ApiItem = serde_json::from_slice(create_res.body()).unwrap();

            // Act.
            let res = warp::test::request()
                .method("GET")
                .path(&format!("/items/{}", created.id.as_ref().unwrap()))
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::OK);
            let found: ApiItem = serde_json::from_slice(res.body()).unwrap();
            assert_eq!(created, found);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_nonexisting() {
        test_filter(|filter| async move {
            // Act.
            let id = ApiItem::new_id().unwrap();
            let res = warp::test::request()
                .method("GET")
                .path(&format!("/items/{}", id))
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_bad_id() {
        test_filter(|filter| async move {
            // Act.
            let id = "abc";
            let res = warp::test::request()
                .method("GET")
                .path(&format!("/items/{}", id))
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::NOT_FOUND);
        })
        .await
    }

    #[tokio::test]
    async fn test_list() {
        test_filter(|filter| async move {
            // Arrange.
            let items = vec![
                ApiItem {
                    id: None,
                    name: crockford::random_id(),
                },
                ApiItem {
                    id: None,
                    name: crockford::random_id(),
                },
            ];
            for item in &items {
                warp::test::request()
                    .method("POST")
                    .path("/items")
                    .body(serde_json::to_string(&item).unwrap())
                    .reply(&filter)
                    .await;
            }

            // Act.
            let res = warp::test::request()
                .method("GET")
                .path("/items")
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::OK);
            let list: Vec<ApiItem> = serde_json::from_slice(res.body()).unwrap();
            assert_eq!(list.len(), items.len());
            for found in &list {
                assert!(items.iter().any(|item| found.name == item.name));
                assert!(!found.id.as_ref().unwrap().is_empty());
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_update_forbidden() {
        test_filter(|filter| async move {
            // Arrange.
            let item = ApiItem {
                id: ApiItem::new_id(),
                name: crockford::random_id(),
            };

            // Act.
            let res = warp::test::request()
                .method("PATCH")
                .path(&format!("/items/{}", item.id.as_ref().unwrap()))
                .body(serde_json::to_string(&item).unwrap())
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::FORBIDDEN);
        })
        .await
    }

    #[tokio::test]
    async fn test_replace() {
        test_filter(|filter| async move {
            // Arrange.
            let item = ApiItem {
                id: None,
                name: crockford::random_id(),
            };
            let create_res = warp::test::request()
                .method("POST")
                .path("/items")
                .body(serde_json::to_string(&item).unwrap())
                .reply(&filter)
                .await;
            let created: ApiItem = serde_json::from_slice(create_res.body()).unwrap();
            let replacement = ApiItem {
                id: None,
                name: crockford::random_id(),
            };

            // Act.
            let res = warp::test::request()
                .method("PUT")
                .path(&format!("/items/{}", created.id.as_ref().unwrap()))
                .body(serde_json::to_string(&replacement).unwrap())
                .reply(&filter)
                .await;

            // Assert.
            assert_eq!(res.status(), StatusCode::OK);
            let res_item: ApiItem = serde_json::from_slice(res.body()).unwrap();
            assert_eq!(res_item.id, created.id);
            assert_eq!(res_item.name, replacement.name);
            let found_res = warp::test::request()
                .method("GET")
                .path(&format!("/items/{}", created.id.as_ref().unwrap()))
                .reply(&filter)
                .await;
            let found_item: ApiItem = serde_json::from_slice(found_res.body()).unwrap();
            assert_eq!(found_item.id, created.id);
            assert_eq!(found_item.name, replacement.name);
        })
        .await
    }

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
    pub struct MemItemRepo {
        pub collection: mem_op::Collection<RepoItem>,
    }

    impl Default for MemItemRepo {
        fn default() -> Self {
            Self {
                collection: Default::default(),
            }
        }
    }

    impl ItemRepo for MemItemRepo {}

    #[async_trait::async_trait]
    impl InsertOne<RepoItem> for MemItemRepo {
        async fn insert_one(&self, item: &RepoItem) -> anyhow::Result<()> {
            mem_op::insert_one(item, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl ReplaceOne<RepoItem> for MemItemRepo {
        async fn replace_one(&self, item: &RepoItem) -> ReplaceResult {
            mem_op::replace_one(item, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl FindOne<RepoItem> for MemItemRepo {
        async fn find_one(&self, id: &<RepoItem as HasId>::Id) -> anyhow::Result<Option<RepoItem>> {
            mem_op::find_one(id, &self.collection)
        }
    }

    #[async_trait::async_trait]
    impl Find<RepoItem> for MemItemRepo {
        async fn find(&self) -> anyhow::Result<Box<dyn ItemStream<RepoItem>>> {
            mem_op::find(&self.collection)
        }
    }

    #[async_trait::async_trait]
    impl DeleteOne<RepoItem> for MemItemRepo {
        async fn delete_one(&self, id: &<RepoItem as HasId>::Id) -> DeleteResult {
            mem_op::delete_one(id, &self.collection)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ApiItem {
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

    #[derive(Clone)]
    pub struct ItemApi {
        pub repo: Arc<dyn ItemRepo + Send + Sync + 'static>,
    }

    impl ItemApi {
        pub fn new(repo: impl ItemRepo + Send + Sync + 'static) -> Self {
            Self {
                repo: Arc::new(repo),
            }
        }
    }

    impl ResourceApi for ItemApi {
        fn collection_name(&self) -> String {
            "items".to_string()
        }

        fn get(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
            get::<ApiItem, _, _>(self.collection_name(), self.repo.clone())
        }

        fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
            list::<ApiItem, _, _>(self.collection_name(), self.repo.clone())
        }

        fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
            create::<ApiItem, _, _>(self.collection_name(), self.repo.clone())
        }

        fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
            delete(self.collection_name(), self.repo.clone())
        }

        fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
            replace::<ApiItem, _, _>(self.collection_name(), self.repo.clone())
        }
    }

    async fn test_filter<T, F>(test: T)
    where
        T: Fn(BoxedFilter<(Box<dyn Reply>,)>) -> F,
        F: Future,
    {
        let repo = MemItemRepo::default();
        let api = ItemApi::new(repo);
        let filter = api.all();
        test(filter).await;
    }
}
