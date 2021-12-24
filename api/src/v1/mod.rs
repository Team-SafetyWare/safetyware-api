use crate::db;
use crate::repo::company::CompanyRepo;
use crate::warp_ext;
use crate::warp_ext::IntoInfallible;
use mongodb::Database;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

mod companies;

pub fn filter(db: Database, company_repo: impl CompanyRepo) -> BoxedFilter<(impl Reply,)> {
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
