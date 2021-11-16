mod settings;

use crate::settings::Settings;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use mongodb::{Client, Database};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::env;
use std::net::Ipv4Addr;
use warp::Filter;

#[derive(Serialize, Deserialize)]
struct ViewCount {
    count: i64,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let settings = Settings::read();
    let db = Client::with_uri_str(&settings.db_uri)
        .await
        .unwrap()
        .database("sw");

    let count = warp::any()
        .and(with_clone(db))
        .and_then(move |db: Database| async move {
            let view_count = view_count(&db).await;
            Result::<_, Infallible>::Ok(format!("View count: {}", view_count))
        });

    let health = warp::path("v1").and(warp::path("health")).map(warp::reply);

    let route = health.or(count).with(warp::log("api"));
    let port = get_port();

    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, port)).await;
}

pub fn with_clone<T: Clone + Send>(
    item: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || item.clone())
}

pub fn get_port() -> u16 {
    // When running as an Azure Function use the supplied port, otherwise use the default.
    match env::var("FUNCTIONS_CUSTOMHANDLER_PORT") {
        Ok(port) => port.parse().expect("Custom Handler port is not a number"),
        Err(_) => 3001,
    }
}

pub async fn view_count(db: &Database) -> i64 {
    let coll = db.collection("view_count");
    let view_count: ViewCount = coll
        .find_one_and_update(
            bson::doc! {},
            bson::doc! { "$inc": { "count": 1i64 } },
            Some(
                FindOneAndUpdateOptions::builder()
                    .upsert(true)
                    .return_document(Some(ReturnDocument::After))
                    .build(),
            ),
        )
        .await
        .unwrap()
        .unwrap();
    view_count.count
}
