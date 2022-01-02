use crate::repo::company::CompanyRepo;
use crate::repo::{company, location_reading, person};

use crate::warp_ext;
use crate::warp_ext::BoxReply;
use derive_more::From;
use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode};

use crate::repo::location_reading::LocationReadingRepo;
use crate::repo::person::PersonRepo;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::Response;
use warp::{Filter, Reply};

pub fn graphql_filter(context: Context) -> BoxedFilter<(impl Reply,)> {
    let state = warp_ext::with_clone(context).boxed();
    let schema = schema();
    (warp::get().or(warp::post()).unify())
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(schema, state))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

pub fn graphiql_filter() -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path("graphiql"))
        .and(juniper_warp::graphiql_filter("/graphql", None))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}

#[derive(Clone)]
pub struct Context {
    pub company_repo: Arc<dyn CompanyRepo + Send + Sync + 'static>,
    pub person_repo: Arc<dyn PersonRepo + Send + Sync + 'static>,
    pub location_reading_repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
}

impl juniper::Context for Context {}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn get_company(#[graphql(context)] context: &Context, id: String) -> Option<Company> {
        context
            .company_repo
            .find_one(&id)
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn get_companies(#[graphql(context)] context: &Context) -> Vec<Company> {
        context
            .company_repo
            .find()
            .await
            .unwrap()
            .map_ok(Into::into)
            .try_collect()
            .await
            .unwrap()
    }

    async fn get_person(#[graphql(context)] context: &Context, id: String) -> Option<Person> {
        context
            .person_repo
            .find_one(&id)
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn get_people(#[graphql(context)] context: &Context) -> Vec<Person> {
        context
            .person_repo
            .find()
            .await
            .unwrap()
            .map_ok(Into::into)
            .try_collect()
            .await
            .unwrap()
    }

    async fn get_location_readings(#[graphql(context)] context: &Context) -> Vec<LocationReading> {
        context
            .location_reading_repo
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

#[graphql_object(context = Context)]
impl Company {
    fn id(&self) -> &str {
        &self.0.id
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    async fn people(&self, context: &Context) -> Vec<Person> {
        context
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

#[graphql_object(context = Context)]
impl Person {
    fn id(&self) -> &str {
        &self.0.id
    }

    fn name(&self) -> &str {
        &self.0.name
    }

    async fn company(&self, context: &Context) -> Option<Company> {
        context
            .company_repo
            .find_one(&self.0.company_id)
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn location_readings(&self, context: &Context) -> Vec<LocationReading> {
        context
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

#[graphql_object(context = Context)]
impl LocationReading {
    fn timestamp(&self) -> &DateTime<Utc> {
        &self.0.timestamp
    }

    async fn person(&self, context: &Context) -> Option<Person> {
        context
            .person_repo
            .find_one(&self.0.person_id)
            .await
            .unwrap()
            .map(Into::into)
    }

    fn coordinates(&self) -> &Vec<f64> {
        &self.0.coordinates
    }
}
