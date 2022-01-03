pub mod company;
pub mod location_reading;
pub mod person;

use crate::graphql::company::{Company, CompanyInput};
use crate::graphql::location_reading::LocationReading;
use crate::graphql::person::{Person, PersonInput};
use crate::repo::company::CompanyRepo;
use crate::repo::location_reading::LocationReadingRepo;
use crate::repo::person::PersonRepo;
use crate::warp_ext;
use crate::warp_ext::BoxReply;
use juniper::{graphql_object, EmptySubscription, RootNode, ID};
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

type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
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
    async fn company(#[graphql(context)] context: &Context, id: ID) -> Option<Company> {
        company::company(context, id).await
    }

    async fn companies(#[graphql(context)] context: &Context) -> Vec<Company> {
        company::companies(context).await
    }

    async fn person(#[graphql(context)] context: &Context, id: ID) -> Option<Person> {
        person::person(context, id).await
    }

    async fn people(#[graphql(context)] context: &Context) -> Vec<Person> {
        person::people(context).await
    }

    async fn location_readings(#[graphql(context)] context: &Context) -> Vec<LocationReading> {
        location_reading::location_readings(context).await
    }
}

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn create_company(#[graphql(context)] context: &Context, input: CompanyInput) -> Company {
        company::create_company(context, input).await
    }

    async fn replace_company(
        #[graphql(context)] context: &Context,
        id: ID,
        input: CompanyInput,
    ) -> Company {
        company::replace_company(context, id, input).await
    }

    async fn delete_company(#[graphql(context)] context: &Context, id: ID) -> ID {
        company::delete_company(context, id).await
    }

    async fn create_person(#[graphql(context)] context: &Context, input: PersonInput) -> Person {
        person::create_person(context, input).await
    }

    async fn replace_person(
        #[graphql(context)] context: &Context,
        id: ID,
        input: PersonInput,
    ) -> Person {
        person::replace_person(context, id, input).await
    }

    async fn delete_person(#[graphql(context)] context: &Context, id: ID) -> ID {
        person::delete_person(context, id).await
    }
}
