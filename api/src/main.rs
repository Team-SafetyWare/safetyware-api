pub mod crockford;
pub mod db;
pub mod graphql;
pub mod repo;
pub mod settings;
pub mod warp_ext;

use crate::graphql::Context;
use crate::repo::company::{CompanyRepo, MongoCompanyRepo};
use crate::repo::location_reading::{LocationReadingRepo, MongoLocationReadingRepo};
use crate::repo::person::{MongoPersonRepo, PersonRepo};
use crate::repo::user_account::{MongoUserAccountRepo, UserAccountRepo};
use crate::settings::Settings;
use mongodb::Database;
use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;
use warp::cors::Cors;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

const DEFAULT_PORT: u16 = 3001;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let settings = Settings::read();
    let db = db::connect_and_prepare(&settings.db_uri).await?;
    let company_repo = MongoCompanyRepo::new(db.clone());
    let person_repo = MongoPersonRepo::new(db.clone());
    let location_reading_repo = MongoLocationReadingRepo::new(db.clone());
    let user_account_repo = MongoUserAccountRepo::new(db.clone());
    let route = filter(
        db,
        company_repo,
        person_repo,
        location_reading_repo,
        user_account_repo,
    )
    .with(log())
    .with(cors());
    let port = get_port();
    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, port)).await;
    Ok(())
}

fn filter(
    db: Database,
    company_repo: impl CompanyRepo + Clone + Send + Sync + 'static,
    person_repo: impl PersonRepo + Clone + Send + Sync + 'static,
    location_reading_repo: impl LocationReadingRepo + Clone + Send + Sync + 'static,
    user_account_repo: impl UserAccountRepo + Clone + Send + Sync + 'static,
) -> BoxedFilter<(impl Reply,)> {
    let graphql = graphql::graphql_filter(Context {
        company_repo: Arc::new(company_repo),
        person_repo: Arc::new(person_repo),
        location_reading_repo: Arc::new(location_reading_repo),
        user_account_repo: Arc::new(user_account_repo),
    });
    let graphiql = graphql::graphiql_filter();
    let robots = robots();
    let health = health(db);
    graphql.or(graphiql).or(robots).or(health).boxed()
}

fn robots() -> BoxedFilter<(impl Reply,)> {
    warp::path("robots.txt")
        .map(|| "User-agent: *\nDisallow: /")
        .boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .then(move |db: Database| async move {
            db::test_connection(&db).await?;
            Ok(warp::reply())
        })
        .map(warp_ext::convert_err)
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
