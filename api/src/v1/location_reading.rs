use crate::v1::ResourceApi;
use crate::warp_ext;
use crate::warp_ext::AsJsonReply;
use chrono::{DateTime, Utc};

use futures_util::TryStreamExt;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};

use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLocationReading {
    pub timestamp: String,
    pub person_id: String,
    pub coordinates: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbLocationReading {
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub metadata: Metadata,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub person_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub coordinates: Vec<f64>,
}

impl From<DbLocationReading> for ApiLocationReading {
    fn from(value: DbLocationReading) -> Self {
        Self {
            person_id: value.metadata.person_id,
            timestamp: value.timestamp.to_string(),
            coordinates: value.location.coordinates,
        }
    }
}

#[derive(Clone)]
pub struct LocationReadingApi {
    pub db: Database,
}

impl LocationReadingApi {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn collection(&self) -> Collection<DbLocationReading> {
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
            .then(
                move |collection: Collection<DbLocationReading>| async move {
                    let all: Vec<ApiLocationReading> = collection
                        .find(None, None)
                        .await?
                        .map_ok(Into::into)
                        .try_collect()
                        .await?;
                    Ok(all.as_json_reply())
                },
            )
            .map(warp_ext::convert_err)
            .boxed()
    }
}
