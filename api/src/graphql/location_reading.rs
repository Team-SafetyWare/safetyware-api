use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::location_reading;
use chrono::{DateTime, Utc};
use derive_more::From;
use futures_util::TryStreamExt;

#[derive(Clone, From)]
pub struct LocationReading(location_reading::LocationReading);

#[juniper::graphql_object(context = Context)]
impl LocationReading {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.0.timestamp
    }

    pub async fn person(&self, context: &Context) -> Option<Person> {
        context
            .person_repo
            .find_one(&self.0.person_id)
            .await
            .unwrap()
            .map(Into::into)
    }

    pub fn coordinates(&self) -> &Vec<f64> {
        &self.0.coordinates
    }
}

pub async fn location_readings(context: &Context) -> Vec<LocationReading> {
    context
        .location_reading_repo
        .find()
        .await
        .unwrap()
        .map_ok(Into::into)
        .try_collect()
        .await
        .unwrap()
}
