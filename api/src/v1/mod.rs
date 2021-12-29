use crate::db;
use crate::repo::company::CompanyRepo;
use crate::repo::person::PersonRepo;
use crate::v1::companies::CompanyApi;
use crate::v1::location_reading::LocationReadingApi;
use crate::v1::people::PersonApi;
use crate::warp_ext;
use mongodb::Database;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

pub mod companies;
pub mod location_reading;
pub mod op;
pub mod people;

pub fn all(
    db: Database,
    company_repo: impl CompanyRepo + Send + Sync + 'static,
    person_repo: impl PersonRepo + Send + Sync + 'static,
) -> BoxedFilter<(impl Reply,)> {
    let company = CompanyApi::new(company_repo).all();
    let person = PersonApi::new(person_repo).all();
    let location_reading = LocationReadingApi::new(db.clone()).all();

    warp::path("v1")
        .and(health(db).or(company).or(person).or(location_reading))
        .boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .then(move |db: Database| async move {
            db::test_connection(&db).await?;
            Ok(warp::reply())
        })
        .map(warp_ext::convert_err)
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
        warp::path(self.collection_name())
            .and(warp::get())
            .and(warp::path::param())
            .map(|_: String| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::get())
            .map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::post())
            .map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }

    fn update(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::patch())
            .map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::delete())
            .map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }

    fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::put())
            .map(|| Box::new(StatusCode::FORBIDDEN) as Box<dyn Reply>)
            .boxed()
    }
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
