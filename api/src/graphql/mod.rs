pub mod company;
pub mod device;
pub mod gas_reading;
pub mod incident;
pub mod incident_stats;
pub mod location_reading;
pub mod person;
pub mod team;
pub mod user_account;

use crate::auth::{AuthProvider, Claims, ClaimsProvider};
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
use crate::repo::user_account::{Access, ArcUserAccountRepo};
use crate::warp_ext;
use crate::warp_ext::BoxReply;
use juniper::{graphql_object, EmptySubscription, FieldError, FieldResult, RootNode, ID};
use warp::filters::BoxedFilter;
use warp::http::header::AUTHORIZATION;
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
    pub claims_provider: ClaimsProvider,
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
    pub claims_provider: ClaimsProvider,
}

impl juniper::Context for Context {}

pub fn graphql_filter(deps: Deps) -> BoxedFilter<(Box<dyn Reply>,)> {
    let state = state_filter(deps);
    let schema = schema();
    (warp::get().or(warp::post()).unify())
        .and(warp::path("graphql"))
        .and(juniper_warp::make_graphql_filter(schema, state))
        .map(|r: Response<Vec<u8>>| r.boxed())
        .boxed()
}

pub fn state_filter(deps: Deps) -> BoxedFilter<(Context,)> {
    // Todo: Extract claims on each request.
    claims_filter(deps.claims_provider.clone())
        .and(warp_ext::with_clone(deps))
        .map(|claims: Option<Claims>, deps: Deps| create_context(deps, claims))
        .boxed()
}

pub fn claims_filter(claims_provider: ClaimsProvider) -> BoxedFilter<(Option<Claims>,)> {
    warp::header(AUTHORIZATION.as_str())
        .and(warp_ext::with_clone(claims_provider))
        .and_then(
            |token: String, claims_provider: ClaimsProvider| async move {
                let token = token.trim_start_matches("Bearer ");
                let res = claims_provider.verify_token(token);
                match res {
                    Ok(claims) => Ok(Some(claims)),
                    Err(e) => {
                        log::warn!("Invalid token: {}", e);
                        Err(warp::reject())
                    }
                }
            },
        )
        .or(warp::any().map(|| None))
        .unify()
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
        claims_provider: deps.claims_provider,
    }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn company(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<Company>> {
        verify_view(&context.claims)?;
        company::get(context, id).await
    }

    async fn companies(#[graphql(context)] context: &Context) -> FieldResult<Vec<Company>> {
        verify_view(&context.claims)?;
        company::list(context).await
    }

    async fn device(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Device>> {
        verify_view(&context.claims)?;
        device::get(context, id).await
    }

    async fn devices(#[graphql(context)] context: &Context) -> FieldResult<Vec<Device>> {
        verify_view(&context.claims)?;
        device::list(context).await
    }

    async fn gas_readings(
        #[graphql(context)] context: &Context,
        filter: Option<GasReadingFilter>,
    ) -> FieldResult<Vec<GasReading>> {
        verify_view(&context.claims)?;
        gas_reading::list(context, filter).await
    }

    async fn incident(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<Incident>> {
        verify_view(&context.claims)?;
        incident::get(context, id).await
    }

    async fn incidents(
        #[graphql(context)] context: &Context,
        filter: Option<IncidentFilter>,
    ) -> FieldResult<Vec<Incident>> {
        verify_view(&context.claims)?;
        incident::list(context, filter).await
    }

    async fn incident_stats(
        #[graphql(context)] context: &Context,
        filter: Option<IncidentStatsFilter>,
    ) -> FieldResult<Vec<IncidentStats>> {
        verify_view(&context.claims)?;
        incident_stats::list(context, filter).await
    }

    async fn location_readings(
        #[graphql(context)] context: &Context,
        filter: Option<LocationReadingFilter>,
    ) -> FieldResult<Vec<LocationReading>> {
        verify_view(&context.claims)?;
        location_reading::list(context, filter).await
    }

    async fn person(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Person>> {
        verify_view(&context.claims)?;
        person::get(context, id).await
    }

    async fn people(#[graphql(context)] context: &Context) -> FieldResult<Vec<Person>> {
        verify_view(&context.claims)?;
        person::list(context).await
    }

    async fn team(#[graphql(context)] context: &Context, id: ID) -> FieldResult<Option<Team>> {
        verify_view(&context.claims)?;
        team::get(context, id).await
    }

    async fn teams(#[graphql(context)] context: &Context) -> FieldResult<Vec<Team>> {
        verify_view(&context.claims)?;
        team::list(context).await
    }

    async fn user_account(
        #[graphql(context)] context: &Context,
        id: ID,
    ) -> FieldResult<Option<UserAccount>> {
        verify_view(&context.claims)?;
        user_account::get(context, id).await
    }

    async fn user_accounts(#[graphql(context)] context: &Context) -> FieldResult<Vec<UserAccount>> {
        verify_view(&context.claims)?;
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
        verify_admin(&context.claims)?;
        company::create(context, input).await
    }

    async fn replace_company(
        #[graphql(context)] context: &Context,
        id: ID,
        input: CompanyInput,
    ) -> FieldResult<Company> {
        verify_admin(&context.claims)?;
        company::replace(context, id, input).await
    }

    async fn delete_company(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
        company::delete(context, id).await
    }

    async fn create_device(
        #[graphql(context)] context: &Context,
        input: DeviceInput,
    ) -> FieldResult<Device> {
        verify_admin(&context.claims)?;
        device::create(context, input).await
    }

    async fn replace_device(
        #[graphql(context)] context: &Context,
        input: DeviceInput,
    ) -> FieldResult<Device> {
        verify_admin(&context.claims)?;
        device::replace(context, input).await
    }

    async fn delete_device(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
        device::delete(context, id).await
    }

    async fn create_incident(
        #[graphql(context)] context: &Context,
        input: IncidentInput,
    ) -> FieldResult<Incident> {
        verify_admin(&context.claims)?;
        incident::create(context, input).await
    }

    async fn replace_incident(
        #[graphql(context)] context: &Context,
        id: ID,
        input: IncidentInput,
    ) -> FieldResult<Incident> {
        verify_admin(&context.claims)?;
        incident::replace(context, id, input).await
    }

    async fn delete_incident(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
        incident::delete(context, id).await
    }

    async fn create_person(
        #[graphql(context)] context: &Context,
        input: PersonInput,
    ) -> FieldResult<Person> {
        verify_admin(&context.claims)?;
        person::create(context, input).await
    }

    async fn replace_person(
        #[graphql(context)] context: &Context,
        id: ID,
        input: PersonInput,
    ) -> FieldResult<Person> {
        verify_admin(&context.claims)?;
        person::replace(context, id, input).await
    }

    async fn delete_person(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
        person::delete(context, id).await
    }

    async fn create_team(
        #[graphql(context)] context: &Context,
        input: TeamInput,
    ) -> FieldResult<Team> {
        verify_admin(&context.claims)?;
        team::create(context, input).await
    }

    async fn delete_team(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
        team::delete(context, id).await
    }

    pub async fn team_add_person(
        context: &Context,
        team_id: ID,
        person_id: ID,
    ) -> FieldResult<Option<Team>> {
        verify_admin(&context.claims)?;
        team::add_person(context, team_id, person_id).await
    }

    pub async fn team_remove_person(
        context: &Context,
        team_id: ID,
        person_id: ID,
    ) -> FieldResult<Option<Team>> {
        verify_admin(&context.claims)?;
        team::remove_person(context, team_id, person_id).await
    }

    async fn create_user_account(
        #[graphql(context)] context: &Context,
        input: UserAccountInput,
    ) -> FieldResult<UserAccount> {
        verify_admin(&context.claims)?;
        user_account::create(context, input).await
    }

    async fn replace_user_account(
        #[graphql(context)] context: &Context,
        id: ID,
        input: UserAccountInput,
    ) -> FieldResult<UserAccount> {
        verify_admin(&context.claims)?;
        user_account::replace(context, id, input).await
    }

    async fn delete_user_account(#[graphql(context)] context: &Context, id: ID) -> FieldResult<ID> {
        verify_admin(&context.claims)?;
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
        verify_admin(&context.claims)?;
        user_account::set_password(context, user_account_id, password).await
    }

    async fn set_user_account_profile_image(
        #[graphql(context)] context: &Context,
        user_account_id: ID,
        image_base64: String,
    ) -> FieldResult<String> {
        verify_admin(&context.claims)?;
        user_account::set_profile_image(context, user_account_id, image_base64).await
    }
}

fn verify_view(claims: &Option<Claims>) -> Result<(), FieldError> {
    // Any authenticated user has at least view authorization.
    if claims.is_some() {
        Ok(())
    } else {
        Err(unauthorized_error())
    }
}

fn verify_admin(claims: &Option<Claims>) -> Result<(), FieldError> {
    if matches!(claims.as_ref().map(|c| c.access), Some(Access::Admin)) {
        Ok(())
    } else {
        Err(unauthorized_error())
    }
}

fn unauthorized_error() -> FieldError {
    anyhow::Error::msg("Unauthorized").into()
}
