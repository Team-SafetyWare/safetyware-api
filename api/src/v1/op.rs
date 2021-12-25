use crate::common::{HasId, NewId, SetId};
use crate::repo::op::{Find, FindOne, InsertOne};
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, BoxReplyInfallible};
use futures_util::TryStreamExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::TryInto;
use std::fmt::Debug;
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
    RepoItem::Id: FromStr + Send,
    <RepoItem::Id as FromStr>::Err: Debug,
{
    warp::path(collection_name)
        .and(warp::get())
        .and(warp_ext::with_clone(repo))
        .and(warp::path::param())
        .and_then(move |repo: Arc<Repo>, id_string: String| async move {
            let id = RepoItem::Id::from_str(&id_string).unwrap();
            let item = repo.find_one(id).await.unwrap().map(Item::from);
            match item {
                None => StatusCode::NOT_FOUND.boxed_infallible(),
                Some(item) => item.as_json_reply().boxed_infallible(),
            }
        })
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
        .and_then(move |repo: Arc<Repo>| async move {
            repo.find()
                .await
                .unwrap()
                .map_ok(Into::into)
                .try_collect::<Vec<Item>>()
                .await
                .unwrap()
                .as_json_reply()
                .boxed_infallible()
        })
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
    <Item as TryInto<RepoItem>>::Error: Debug + Send,
    RepoItem: Send,
{
    warp::path(collection_name)
        .and(warp::post())
        .and(warp_ext::with_clone(repo))
        .and(warp::body::json())
        .and_then(move |repo: Arc<Repo>, mut item: Item| async move {
            item.set_id(Item::new_id());
            let repo_item = item.clone().try_into().unwrap();
            let fut = repo.insert_one(&repo_item);
            fut.await.unwrap();
            item.as_json_reply().boxed_infallible()
        })
        .boxed()
}