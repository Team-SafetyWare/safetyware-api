use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::gas_reading;
use crate::repo::gas_reading::GasReadingFilter as RepoGasReadingFilter;
use chrono::{DateTime, Utc};
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::FieldResult;

#[derive(Clone, From)]
pub struct GasReading(pub gas_reading::GasReading);

#[derive(juniper::GraphQLInputObject, Default)]
pub struct GasReadingFilter {
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[juniper::graphql_object(context = Context)]
impl GasReading {
    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.0.timestamp
    }

    pub async fn person(&self, context: &Context) -> FieldResult<Option<Person>> {
        Ok(context
            .person_repo
            .find_one(&self.0.person_id)
            .await?
            .map(Into::into))
    }

    pub fn gas(&self) -> &str {
        &self.0.gas
    }

    pub fn density(&self) -> f64 {
        self.0.density
    }

    pub fn coordinates(&self) -> &Vec<f64> {
        &self.0.coordinates
    }
}

pub async fn list(
    context: &Context,
    filter: Option<GasReadingFilter>,
) -> FieldResult<Vec<GasReading>> {
    let filter = filter.unwrap_or_default();
    let mut vec: Vec<GasReading> = context
        .gas_reading_repo
        .find(&RepoGasReadingFilter {
            person_ids: None,
            min_timestamp: filter.min_timestamp,
            max_timestamp: filter.max_timestamp,
        })
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?;
    vec.sort_by_key(|l| l.0.timestamp);
    Ok(vec)
}
