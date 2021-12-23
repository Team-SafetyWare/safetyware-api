use mongodb::{Client, Database};
use std::sync::Mutex;

mod company;

lazy_static::lazy_static! {
    static ref DB: Mutex<Option<Database>> = Mutex::new(None);
}

pub async fn db() -> Database {
    let mut db_static = DB.lock().unwrap();
    if let Some(db) = &*db_static {
        db.clone()
    } else {
        let db = Client::with_uri_str("mongodb://localhost:42781")
            .await
            .unwrap()
            .database("sw");
        *db_static = Some(db.clone());
        db
    }
}
