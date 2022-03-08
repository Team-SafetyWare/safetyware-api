use crate::repo::user_account::ArcUserAccountRepo;
use crate::warp_ext;
use mongodb::Database;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

#[derive(Clone)]
pub struct Context {
    pub user_account_repo: ArcUserAccountRepo,
    pub db: Database,
}

pub fn v1(context: Context) -> BoxedFilter<(impl Reply,)> {
    warp::path("v1").and(health(context.db)).boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .then(move |db: Database| async move {
            crate::db::test_connection(&db).await?;
            Ok(warp::reply())
        })
        .map(warp_ext::convert_err)
        .boxed()
}
