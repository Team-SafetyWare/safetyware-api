use crate::repo::location_reading::LocationReading as RepoLocationReading;
use crate::v1::{op, ResourceApi};

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::repo::location_reading::LocationReadingRepo;
use warp::filters::BoxedFilter;
use warp::Reply;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationReading {
    pub timestamp: String,
    pub person_id: String,
    pub coordinates: Vec<f64>,
}

impl From<RepoLocationReading> for LocationReading {
    fn from(value: RepoLocationReading) -> Self {
        Self {
            timestamp: value.timestamp,
            person_id: value.person_id,
            coordinates: value.coordinates,
        }
    }
}

#[derive(Clone)]
pub struct LocationReadingApi {
    pub repo: Arc<dyn LocationReadingRepo + Send + Sync + 'static>,
}

impl LocationReadingApi {
    pub fn new(repo: impl LocationReadingRepo + Send + Sync + 'static) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }
}

impl ResourceApi for LocationReadingApi {
    fn collection_name(&self) -> String {
        "locationReadings".to_string()
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::list::<LocationReading, _, _>(self.collection_name(), self.repo.clone())
    }
}
