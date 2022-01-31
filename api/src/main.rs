pub mod blackline;
pub mod crockford;
pub mod db;
pub mod graphql;
pub mod repo;
pub mod settings;
pub mod warp_ext;

use crate::blackline::import;
use crate::graphql::Context;
use crate::repo::company::MongoCompanyRepo;
use crate::repo::device::MongoDeviceRepo;
use crate::repo::location_reading::MongoLocationReadingRepo;
use crate::repo::person::MongoPersonRepo;
use crate::repo::user_account::MongoUserAccountRepo;
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
    let graphql_context = Context {
        company_repo: Arc::new(MongoCompanyRepo::new(db.clone())),
        device_repo: Arc::new(MongoDeviceRepo::new(db.clone())),
        location_reading_repo: Arc::new(MongoLocationReadingRepo::new(db.clone())),
        person_repo: Arc::new(MongoPersonRepo::new(db.clone())),
        user_account_repo: Arc::new(MongoUserAccountRepo::new(db.clone())),
    };
    let import_device_data_context = import::DeviceDataContext {
        device_repo: graphql_context.device_repo.clone(),
        location_reading_repo: graphql_context.location_reading_repo.clone(),
    };
    let route = filter(db, graphql_context, import_device_data_context)
        .with(log())
        .with(cors());
    let port = get_port();
    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, port)).await;
    Ok(())
}

fn filter(
    db: Database,
    graphql_context: Context,
    import_device_data_context: import::DeviceDataContext,
) -> BoxedFilter<(impl Reply,)> {
    graphql::graphql_filter(graphql_context)
        .or(graphql::graphiql_filter())
        .or(import::device_data_filter(import_device_data_context))
        .or(health(db))
        .or(robots())
        .boxed()
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
