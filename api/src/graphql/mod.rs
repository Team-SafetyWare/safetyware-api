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
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode, ID};
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::Response;
use warp::{Filter, Reply};

pub fn graphql_filter(context: Context) -> BoxedFilter<(Box<dyn Reply>,)> {
    let state = warp_ext::with_clone(context).boxed();
    let schema = schema();
    (warp::get().or(warp::post()).unify())
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(schema, state))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

pub fn graphiql_filter() -> BoxedFilter<(Box<dyn Reply>,)> {
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
    async fn company(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<Company>> {
        company::get(context, id).await
    }

    async fn companies(#[graphql(context)] context: &Context) -> FieldResult<Vec<Company>> {
        company::list(context).await
    }

    async fn person(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Person>> {
        person::get(context, id).await
    }

    async fn people(#[graphql(context)] context: &Context) -> FieldResult<Vec<Person>> {
        person::list(context).await
    }

    async fn location_readings(
        #[graphql(context)] context: &Context,
    ) -> FieldResult<Vec<LocationReading>> {
        location_reading::list(context).await
    }
}

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn create_company(
        #[graphql(context)] context: &Context,
        input: CompanyInput,
    ) -> FieldResult<Company> {
        company::create(context, input).await
    }

    async fn replace_company(
        #[graphql(context)] context: &Context,
        id: ID,
        input: CompanyInput,
    ) -> FieldResult<Company> {
        company::replace(context, id, input).await
    }

    async fn delete_company(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        company::delete(context, id).await
    }

    async fn create_person(
        #[graphql(context)] context: &Context,
        input: PersonInput,
    ) -> FieldResult<Person> {
        person::create(context, input).await
    }

    async fn replace_person(
        #[graphql(context)] context: &Context,
        id: ID,
        input: PersonInput,
    ) -> FieldResult<Person> {
        person::replace(context, id, input).await
    }

    async fn delete_person(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        person::delete(context, id).await
    }
}
