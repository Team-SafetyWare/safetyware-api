pub mod company;
pub mod device;
pub mod gas_reading;
pub mod incident;
pub mod incident_stats;
pub mod location_reading;
pub mod person;
pub mod team;
pub mod user_account;

use crate::auth::{AuthProvider, Claims, TokenProvider};
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
use crate::repo::device::ArcDeviceRepo;
use crate::repo::gas_reading::ArcGasReadingRepo;
use crate::repo::incident::ArcIncidentRepo;
use crate::repo::incident_stats::ArcIncidentStatsRepo;
use crate::repo::location_reading::ArcLocationReadingRepo;
use crate::repo::person::ArcPersonRepo;
use crate::repo::team::ArcTeamRepo;
use crate::repo::user_account::ArcUserAccountRepo;
use crate::warp_ext;
use crate::warp_ext::BoxReply;
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode, ID};
use warp::filters::BoxedFilter;
use warp::http::Response;
use warp::{Filter, Reply};

#[derive(Clone)]
pub struct Deps {
    pub company_repo: ArcCompanyRepo,
    pub device_repo: ArcDeviceRepo,
    pub gas_reading_repo: ArcGasReadingRepo,
    pub incident_repo: ArcIncidentRepo,
    pub incident_stats_repo: ArcIncidentStatsRepo,
    pub location_reading_repo: ArcLocationReadingRepo,
    pub person_repo: ArcPersonRepo,
    pub team_repo: ArcTeamRepo,
    pub user_account_repo: ArcUserAccountRepo,
    pub auth_provider: AuthProvider,
    pub token_provider: TokenProvider,
}

#[derive(Clone)]
pub struct Context {
    pub claims: Option<Claims>,
    pub company_repo: ArcCompanyRepo,
    pub device_repo: ArcDeviceRepo,
    pub gas_reading_repo: ArcGasReadingRepo,
    pub incident_repo: ArcIncidentRepo,
    pub incident_stats_repo: ArcIncidentStatsRepo,
    pub location_reading_repo: ArcLocationReadingRepo,
    pub person_repo: ArcPersonRepo,
    pub team_repo: ArcTeamRepo,
    pub user_account_repo: ArcUserAccountRepo,
    pub auth_provider: AuthProvider,
    pub token_provider: TokenProvider,
}

impl juniper::Context for Context {}

pub fn graphql_filter(deps: Deps) -> BoxedFilter<(Box<dyn Reply>,)> {
    // Todo: Extract claims on each request.
    let context = create_context(deps, None);
    let state = warp_ext::with_clone(context).boxed();
    let schema = schema();
    (warp::get().or(warp::post()).unify())
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(schema, state))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

pub fn playground_filter() -> BoxedFilter<(Box<dyn Reply>,)> {
    warp::get()
        .and(warp::path("playground"))
        .and(juniper_warp::playground_filter("/graphql", None))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}

fn create_context(deps: Deps, claims: Option<Claims>) -> Context {
    Context {
        claims,
        company_repo: deps.company_repo,
        device_repo: deps.device_repo,
        gas_reading_repo: deps.gas_reading_repo,
        incident_repo: deps.incident_repo,
        incident_stats_repo: deps.incident_stats_repo,
        location_reading_repo: deps.location_reading_repo,
        person_repo: deps.person_repo,
        team_repo: deps.team_repo,
        user_account_repo: deps.user_account_repo,
        auth_provider: deps.auth_provider,
        token_provider: deps.token_provider,
    }
}

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

    async fn login(
        #[graphql(context)] context: &Context,
        user_account_id: ID,
        password: String,
    ) -> FieldResult<String> {
        user_account::login(context, user_account_id, password).await
    }

    async fn set_user_account_password(
        #[graphql(context)] context: &Context,
        user_account_id: ID,
        password: String,
    ) -> FieldResult<bool> {
        user_account::set_password(context, user_account_id, password).await
    }

    async fn set_user_account_profile_image(
        #[graphql(context)] context: &Context,
        user_account_id: ID,
        image_base64: String,
    ) -> FieldResult<String> {
        user_account::set_profile_image(context, user_account_id, image_base64).await
    }
}
