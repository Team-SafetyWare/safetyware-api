pub mod auth;
pub mod crockford;
pub mod db;
pub mod graphql;
pub mod image;
pub mod repo;
pub mod rest;
pub mod settings;
pub mod warp_ext;

use crate::auth::{AuthProvider, ClaimsProvider};
use crate::repo::company::MongoCompanyRepo;
use crate::repo::device::MongoDeviceRepo;
use crate::repo::gas_reading::MongoGasReadingRepo;
use crate::repo::incident::MongoIncidentRepo;
use crate::repo::incident_stats::MongoIncidentStatsRepo;
use crate::repo::location_reading::MongoLocationReadingRepo;
use crate::repo::person::MongoPersonRepo;
use crate::repo::team::MongoTeamRepo;
use crate::repo::user_account::MongoUserAccountRepo;
use crate::settings::Settings;
use mongodb::Database;
use std::env;
use std::net::Ipv4Addr;
use warp::cors::Cors;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

const DEFAULT_PORT: u16 = 3001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let settings = Settings::read();
    let db = db::connect_and_prepare(&settings.db_uri).await?;
    let graphql_deps = graphql_deps(db.clone(), &settings.private_key);
    let rest_context = rest_context(db.clone());
    let route = filter(graphql_deps, rest_context).with(log()).with(cors());
    let port = get_port();
    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, port)).await;
    Ok(())
}

fn graphql_deps(db: Database, private_key: &str) -> graphql::Deps {
    graphql::Deps {
        company_repo: MongoCompanyRepo::new(db.clone()).into(),
        device_repo: MongoDeviceRepo::new(db.clone()).into(),
        gas_reading_repo: MongoGasReadingRepo::new(db.clone()).into(),
        incident_repo: MongoIncidentRepo::new(db.clone()).into(),
        incident_stats_repo: MongoIncidentStatsRepo::new(db.clone()).into(),
        location_reading_repo: MongoLocationReadingRepo::new(db.clone()).into(),
        person_repo: MongoPersonRepo::new(db.clone()).into(),
        team_repo: MongoTeamRepo::new(db.clone()).into(),
        user_account_repo: MongoUserAccountRepo::new(db.clone()).into(),
        auth_provider: AuthProvider {
            user_account_repo: MongoUserAccountRepo::new(db).into(),
        },
        claims_provider: ClaimsProvider {
            private_key: private_key.to_string(),
        },
    }
}

fn rest_context(db: Database) -> rest::Context {
    rest::Context {
        user_account_repo: MongoUserAccountRepo::new(db.clone()).into(),
        db,
    }
}

fn filter(graphql_deps: graphql::Deps, rest_context: rest::Context) -> BoxedFilter<(impl Reply,)> {
    graphql::graphql_filter(graphql_deps)
        .or(graphql::playground_filter())
        .or(graphql_doc())
        .or(rest::v1(rest_context))
        .or(robots())
        .boxed()
}

fn graphql_doc() -> BoxedFilter<(impl Reply,)> {
    warp::path("doc").and(warp::fs::dir("doc/public")).boxed()
}

fn robots() -> BoxedFilter<(impl Reply,)> {
    warp::path("robots.txt")
        .map(|| "User-agent: *\nDisallow: /")
        .boxed()
}

fn cors() -> Cors {
    warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Origin",
            "Content-Type",
            "Referer",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
            "User-Agent",
            "Sec-Fetch-Mode",
            "Authorization",
        ])
        .allow_methods(vec!["POST", "GET"])
        .build()
}

fn log() -> warp::log::Log<impl Fn(warp::log::Info) + Copy> {
    warp::log("api")
}

pub fn get_port() -> u16 {
    // When running as an Azure Function use the supplied port, otherwise use the default.
    match env::var("FUNCTIONS_CUSTOMHANDLER_PORT") {
        Ok(port) => port.parse().expect("Custom Handler port is not a number"),
        Err(_) => DEFAULT_PORT,
    }
}
