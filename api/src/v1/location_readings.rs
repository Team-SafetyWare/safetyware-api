use crate::v1::ResourceApi;
use crate::warp_ext;
use crate::warp_ext::AsJsonReply;

use bson::Document;
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReading {
    pub person_id: String,
    pub timestamp: String,
    pub coordinates: Option<String>,
}

#[derive(Clone)]
pub struct LocationReadingApi {
    pub db: Database,
}

impl LocationReadingApi {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn collection(&self) -> Collection<Document> {
        self.db.collection("location_reading")
    }
}

impl ResourceApi for LocationReadingApi {
    fn collection_name(&self) -> String {
        "locationReadings".to_string()
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::get())
            .and(warp_ext::with_clone(self.collection()))
            .then(move |collection: Collection<Document>| async move {
                let all: Vec<_> = collection.find(None, None).await?.try_collect().await?;
                Ok(all.as_json_reply())
            })
            .map(warp_ext::convert_err)
            .boxed()
    }
}
