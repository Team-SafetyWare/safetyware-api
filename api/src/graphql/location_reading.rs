use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo;
use crate::repo::location_reading;
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::FieldResult;

#[derive(Clone, From, Deref, DerefMut)]
pub struct LocationReading(pub location_reading::LocationReading);

#[derive(juniper::GraphQLInputObject, Default)]
pub struct LocationReadingFilter {
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[juniper::graphql_object(context = Context)]
impl LocationReading {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub async fn person(&self, context: &Context) -> FieldResult<Option<Person>> {
        Ok(context
            .person_repo
            .find_one(&self.person_id)
            .await?
            .map(Into::into))
    }

    pub fn coordinates(&self) -> &Vec<f64> {
        &self.coordinates
    }
}

pub async fn list(
    context: &Context,
    filter: Option<LocationReadingFilter>,
) -> FieldResult<Vec<LocationReading>> {
    let filter = filter.unwrap_or_default();
    Ok(context
        .location_reading_repo
        .find(repo::location_reading::LocationReadingFilter {
            person_ids: None,
            min_timestamp: filter.min_timestamp,
            max_timestamp: filter.max_timestamp,
        })
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}
