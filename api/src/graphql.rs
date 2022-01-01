use crate::repo::company;
use crate::repo::company::CompanyRepo;

use crate::warp_ext;
use crate::warp_ext::BoxReply;
use derive_more::From;
use juniper::{
    graphql_object, Context, EmptyMutation,
    EmptySubscription, RootNode,
};


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
    let filter = graphql_filter
        .or(graphiql_filter)
        .unify()
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed();
    filter
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
}

impl Context for Store {}

pub struct Query;

#[graphql_object(context = Store)]
impl Query {
    async fn company(#[graphql(context)] store: &Store, id: String) -> Option<Company> {
        // Todo: Do not unwrap.
        store
            .company_repo
            .find_one(&id)
            .await
            .unwrap()
            .map(Into::into)
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
}
