use crate::db;
use crate::repo::company::CompanyRepo;
use crate::v1::companies::CompanyApi;
use crate::warp_ext;
use mongodb::Database;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

pub mod companies;
pub mod op;

pub fn all(
    db: Database,
    company_repo: impl CompanyRepo + Send + Sync + 'static,
) -> BoxedFilter<(impl Reply,)> {
    let company = CompanyApi {
        repo: Arc::new(company_repo),
    }
    .all();

    warp::path("v1").and(health(db).or(company)).boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .then(move |db: Database| async move {
            db::test_connection(&db).await.unwrap();
            warp::reply()
        })
        .boxed()
}

pub trait ResourceApi {
    fn collection_name(&self) -> String;

    fn all(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        self.get()
            .or(self.list())
            .unify()
            .or(self.create())
            .unify()
            .or(self.update())
            .unify()
            .or(self.delete())
            .unify()
            .or(self.replace())
            .unify()
            .boxed()
    }

    fn get(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::get())
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::get())
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::post())
    }

    fn update(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::patch())
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::delete())
    }

    fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        forbidden_filter(self.collection_name(), warp::put())
    }
}

fn forbidden_filter(
    collection_name: String,
    method: impl Filter<Extract = (), Error = Rejection> + Copy + Send + Sync + 'static,
) -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::path(collection_name)
        .and(method.map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>))
        .boxed()
}

pub trait ResourceOperation {
    fn operation(
        &self,
        method: impl Filter<Extract = (), Error = Rejection> + Copy + Send + Sync + 'static,
    ) -> BoxedFilter<(Self,)>
    where
        Self: Sized;
}

impl<T: ResourceApi + Clone + Send + Sync + 'static> ResourceOperation for T {
    fn operation(
        &self,
        method: impl Filter<Extract = (), Error = Rejection> + Copy + Send + Sync + 'static,
    ) -> BoxedFilter<(Self,)> {
        warp::path(self.collection_name())
            .and(method)
            .and(warp_ext::with_clone(self.clone()))
            .boxed()
    }
}
