mod settings;

use crate::settings::Settings;
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use mongodb::{Client, Database};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
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

    let route = warp::any()
        .and(with_clone(db))
        .and_then(move |db: Database| async move {
            let view_count = view_count(&db).await;
            Result::<_, Infallible>::Ok(format!("View count: {}", view_count))
        });

    warp::serve(route).run((Ipv4Addr::UNSPECIFIED, 3001)).await;
}

pub fn with_clone<T: Clone + Send>(
    item: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || item.clone())
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
