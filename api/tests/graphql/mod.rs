mod company;

use crate::repo;
use api::graphql;
use api::graphql::Context;
use api::repo::company::MongoCompanyRepo;
use api::repo::location_reading::MongoLocationReadingRepo;
use api::repo::person::MongoPersonRepo;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::Reply;

#[derive(Debug, Clone)]
pub struct MongoContext {
    company_repo: MongoCompanyRepo,
    location_reading_repo: MongoLocationReadingRepo,
    person_repo: MongoPersonRepo,
}

async fn test_graphql<T, F>(test: T)
where
    T: Fn(BoxedFilter<(Box<dyn Reply>,)>, MongoContext) -> F,
    F: Future<Output = anyhow::Result<()>>,
{
    let db = repo::new_db().await.unwrap();
    let mc = MongoContext {
        company_repo: MongoCompanyRepo::new(db.clone()),
        location_reading_repo: MongoLocationReadingRepo::new(db.clone()),
        person_repo: MongoPersonRepo::new(db.clone()),
    };
    let context = Context {
        company_repo: Arc::new(mc.company_repo.clone()),
        person_repo: Arc::new(mc.person_repo.clone()),
        location_reading_repo: Arc::new(mc.location_reading_repo.clone()),
    };
    let filter = graphql::graphql_filter(context);
    test(filter, mc).await.unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryJson {
    query: String,
}

fn encode_query(query: String) -> anyhow::Result<String> {
    Ok(serde_json::to_string(&QueryJson { query })?)
}
