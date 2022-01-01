use crate::repo::company::CompanyRepo;
use crate::repo::{company, location_reading, person};

use crate::warp_ext;
use crate::warp_ext::BoxReply;
use derive_more::From;
use juniper::{graphql_object, Context, EmptyMutation, EmptySubscription, RootNode};

use crate::repo::location_reading::LocationReadingRepo;
use crate::repo::person::PersonRepo;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::Response;
use warp::{Filter, Reply};

pub fn filter(store: Store) -> BoxedFilter<(impl Reply,)> {
    let state = warp::any().and(warp_ext::with_clone(store));
    let graphql_filter = (warp::get().or(warp::post()).unify())
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(schema(), state.boxed()));
    let graphiql_filter = warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql", None));
    graphql_filter
        .or(graphiql_filter)
        .unify()
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

type Schema = RootNode<'static, Query, EmptyMutation<Store>, EmptySubscription<Store>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Store>::new(),
        EmptySubscription::<Store>::new(),
    )
}

#[derive(Clone)]
pub struct Store {
    pub company_repo: Arc<dyn CompanyRepo + Send + Sync + 'static>,
    pub person_repo: Arc<dyn PersonRepo + Send + Sync + 'static>,
    pub location_reading_repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
}

impl Context for Store {}

pub struct Query;

#[graphql_object(context = Store)]
impl Query {
    async fn company(#[graphql(context)] store: &Store, id: String) -> Option<Company> {
        store
            .company_repo
            .find_one(&id)
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn companies(#[graphql(context)] store: &Store) -> Vec<Company> {
        store
            .company_repo
            .find()
            .await
            .unwrap()
            .map_ok(Into::into)
            .try_collect()
            .await
            .unwrap()
    }
}

#[derive(Clone, From)]
pub struct Company(company::Company);

#[graphql_object(context = Store)]
impl Company {
    fn id(&self) -> &str {
        &self.0.id
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    async fn people(&self, store: &Store) -> Vec<Person> {
        store
            .person_repo
            .find()
            .await
            .unwrap()
            .try_filter_map(|p| async move {
                Ok(Some(p)
                    .filter(|p| p.company_id == self.0.id)
                    .map(Into::into))
            })
            .try_collect()
            .await
            .unwrap()
    }
}

#[derive(Clone, From)]
pub struct Person(person::Person);

#[graphql_object(context = Store)]
impl Person {
    fn id(&self) -> &str {
        &self.0.id
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    async fn location_readings(&self, store: &Store) -> Vec<LocationReading> {
        store
            .location_reading_repo
            .find()
            .await
            .unwrap()
            .try_filter_map(|lr| async move {
                Ok(Some(lr)
                    .filter(|lr| lr.person_id == self.0.id)
                    .map(Into::into))
            })
            .try_collect()
            .await
            .unwrap()
    }
}

#[derive(Clone, From)]
pub struct LocationReading(location_reading::LocationReading);

#[graphql_object(context = Store)]
impl LocationReading {
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.0.timestamp
    }

    fn coordinates(&self) -> &Vec<f64> {
        &self.0.coordinates
    }
}
