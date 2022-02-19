pub mod company;
pub mod device;
pub mod gas_reading;
pub mod incident;
pub mod incident_stats;
pub mod location_reading;
pub mod person;
pub mod team;
pub mod user_account;

use crate::graphql::company::{Company, CompanyInput};
use crate::graphql::device::Device;
use crate::graphql::device::DeviceInput;
use crate::graphql::gas_reading::{GasReading, GasReadingFilter};
use crate::graphql::incident::{Incident, IncidentFilter, IncidentInput};
use crate::graphql::incident_stats::{IncidentStats, IncidentStatsFilter};
use crate::graphql::location_reading::{LocationReading, LocationReadingFilter};
use crate::graphql::person::{Person, PersonInput};
use crate::graphql::team::{Team, TeamInput};
use crate::graphql::user_account::{UserAccount, UserAccountInput};
use crate::repo::company::ArcCompanyRepo;
use crate::repo::device::DeviceRepo;
use crate::repo::gas_reading::GasReadingRepo;
use crate::repo::incident::IncidentRepo;
use crate::repo::incident_stats::IncidentStatsRepo;
use crate::repo::location_reading::LocationReadingRepo;
use crate::repo::person::PersonRepo;
use crate::repo::team::TeamRepo;
use crate::repo::user_account::UserAccountRepo;
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
    pub company_repo: ArcCompanyRepo,
    pub device_repo: Arc<dyn DeviceRepo + Send + Sync + 'static>,
    pub gas_reading_repo: Arc<dyn GasReadingRepo + Send + Sync + 'static>,
    pub incident_repo: Arc<dyn IncidentRepo + Send + Sync + 'static>,
    pub incident_stats_repo: Arc<dyn IncidentStatsRepo + Send + Sync + 'static>,
    pub location_reading_repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
    pub person_repo: Arc<dyn PersonRepo + Send + Sync + 'static>,
    pub team_repo: Arc<dyn TeamRepo + Send + Sync + 'static>,
    pub user_account_repo: Arc<dyn UserAccountRepo + Send + Sync + 'static>,
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

    async fn device(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Device>> {
        device::get(context, id).await
    }

    async fn devices(#[graphql(context)] context: &Context) -> FieldResult<Vec<Device>> {
        device::list(context).await
    }

    async fn gas_readings(
        #[graphql(context)] context: &Context,
        filter: Option<GasReadingFilter>,
    ) -> FieldResult<Vec<GasReading>> {
        gas_reading::list(context, filter).await
    }

    async fn incident(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<Incident>> {
        incident::get(context, id).await
    }

    async fn incidents(
        #[graphql(context)] context: &Context,
        filter: Option<IncidentFilter>,
    ) -> FieldResult<Vec<Incident>> {
        incident::list(context, filter).await
    }

    async fn incident_stats(
        #[graphql(context)] context: &Context,
        filter: Option<IncidentStatsFilter>,
    ) -> FieldResult<Vec<IncidentStats>> {
        incident_stats::list(context, filter).await
    }

    async fn location_readings(
        #[graphql(context)] context: &Context,
        filter: Option<LocationReadingFilter>,
    ) -> FieldResult<Vec<LocationReading>> {
        location_reading::list(context, filter).await
    }

    async fn person(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Person>> {
        person::get(context, id).await
    }

    async fn people(#[graphql(context)] context: &Context) -> FieldResult<Vec<Person>> {
        person::list(context).await
    }

    async fn team(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Team>> {
        team::get(context, id).await
    }

    async fn teams(#[graphql(context)] context: &Context) -> FieldResult<Vec<Team>> {
        team::list(context).await
    }

    async fn user_account(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<UserAccount>> {
        user_account::get(context, id).await
    }

    async fn user_accounts(#[graphql(context)] context: &Context) -> FieldResult<Vec<UserAccount>> {
        user_account::list(context).await
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

    async fn create_device(
        #[graphql(context)] context: &Context,
        input: DeviceInput,
    ) -> FieldResult<Device> {
        device::create(context, input).await
    }

    async fn replace_device(
        #[graphql(context)] context: &Context,
        input: DeviceInput,
    ) -> FieldResult<Device> {
        device::replace(context, input).await
    }

    async fn delete_device(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        device::delete(context, id).await
    }

    async fn create_incident(
        #[graphql(context)] context: &Context,
        input: IncidentInput,
    ) -> FieldResult<Incident> {
        incident::create(context, input).await
    }

    async fn replace_incident(
        #[graphql(context)] context: &Context,
        id: ID,
        input: IncidentInput,
    ) -> FieldResult<Incident> {
        incident::replace(context, id, input).await
    }

    async fn delete_incident(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        incident::delete(context, id).await
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

    async fn create_team(
        #[graphql(context)] context: &Context,
        input: TeamInput,
    ) -> FieldResult<Team> {
        team::create(context, input).await
    }

    async fn delete_team(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        team::delete(context, id).await
    }

    pub async fn team_add_person(
        context: &Context,
        team_id: ID,
        person_id: ID,
    ) -> FieldResult<Option<Team>> {
        team::add_person(context, team_id, person_id).await
    }

    pub async fn team_remove_person(
        context: &Context,
        team_id: ID,
        person_id: ID,
    ) -> FieldResult<Option<Team>> {
        team::remove_person(context, team_id, person_id).await
    }

    async fn create_user_account(
        #[graphql(context)] context: &Context,
        input: UserAccountInput,
    ) -> FieldResult<UserAccount> {
        user_account::create(context, input).await
    }

    async fn replace_user_account(
        #[graphql(context)] context: &Context,
        id: ID,
        input: UserAccountInput,
    ) -> FieldResult<UserAccount> {
        user_account::replace(context, id, input).await
    }

    async fn delete_user_account(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        user_account::delete(context, id).await
    }
}
