use crate::db;
use crate::repo::company::CompanyRepo;
use crate::warp_ext;
use crate::warp_ext::IntoInfallible;
use mongodb::Database;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

mod companies;

pub fn filter<CR: CompanyRepo>(db: Database, company_repo: CR) -> BoxedFilter<(impl Reply,)> {
    warp::path("v1")
        .and(health(db).or(companies::filter(company_repo)))
        .boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .and_then(move |db: Database| async move {
            db::test_connection(&db).await.unwrap();
            warp::reply().into_infallible()
        })
        .boxed()
}
