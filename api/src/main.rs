pub mod common;
pub mod db;
pub mod repo;
pub mod settings;
pub mod v1;
pub mod warp_ext;

use crate::repo::company::{CompanyRepo, MongoCompanyRepo};
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
    let db = db::connect(&settings.db_uri).await?;
    let company_repo = MongoCompanyRepo::new(db.clone());
    let route = filter(db.clone(), company_repo.clone())
        .with(log())
        .with(cors());
    let port = get_port();
    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, port)).await;
    Ok(())
}

fn filter(
    db: Database,
    company_repo: impl CompanyRepo + Send + Sync + 'static,
) -> BoxedFilter<(impl Reply,)> {
    let v1 = v1::all(db, company_repo);
    let robots = robots();
    v1.or(robots).boxed()
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
