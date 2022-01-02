use crate::repo::company::CompanyRepo;
use crate::repo::{company, location_reading, person};

use crate::warp_ext::BoxReply;
use crate::{crockford, warp_ext};
use derive_more::From;
use juniper::{graphql_object, EmptySubscription, GraphQLInputObject, RootNode, ID};

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
        context
            .company_repo
            .find_one(&id.to_string())
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn companies(#[graphql(context)] context: &Context) -> Vec<Company> {
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

    async fn person(#[graphql(context)] context: &Context, id: ID) -> Option<Person> {
        context
            .person_repo
            .find_one(&id.to_string())
            .await
            .unwrap()
            .map(Into::into)
    }

    async fn people(#[graphql(context)] context: &Context) -> Vec<Person> {
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

    async fn location_readings(#[graphql(context)] context: &Context) -> Vec<LocationReading> {
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

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn create_company(#[graphql(context)] context: &Context, input: CompanyInput) -> Company {
        let item = company::Company {
            id: crockford::random_id(),
            name: input.name,
        };
        context.company_repo.insert_one(&item).await.unwrap();
        item.into()
    }

    async fn replace_company(
        #[graphql(context)] context: &Context,
        id: ID,
        input: CompanyInput,
    ) -> Company {
        let item = company::Company {
            id: id.to_string(),
            name: input.name,
        };
        context.company_repo.replace_one(&item).await.unwrap();
        item.into()
    }
}

#[derive(Clone, From)]
pub struct Company(company::Company);

#[derive(GraphQLInputObject)]
struct CompanyInput {
    name: String,
}

#[graphql_object(context = Context)]
impl Company {
    fn id(&self) -> ID {
        self.0.id.clone().into()
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
    fn id(&self) -> ID {
        self.0.id.clone().into()
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
