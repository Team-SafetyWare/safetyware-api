use crate::common::HasId;
use crate::repo::op::FindOne;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, BoxReplyInfallible};
use serde::Serialize;
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
