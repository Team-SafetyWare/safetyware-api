use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::incident;
use crate::repo::incident::IncidentFilter as RepoIncidentFilter;
use chrono::{DateTime, Utc};
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From)]
pub struct Incident(pub incident::Incident);

#[derive(juniper::GraphQLInputObject, Default)]
pub struct IncidentFilter {
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[juniper::graphql_object(context = Context)]
impl Incident {
    pub fn id(&self) -> ID {
        self.0.id.clone().into()
    }

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

    pub fn coordinates(&self) -> &Vec<f64> {
        &self.0.coordinates
    }

    pub fn r#type(&self) -> &str {
        &self.0.r#type
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Incident>> {
    Ok(context
        .incident_repo
        .find_one(&id.to_string())
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context, filter: Option<IncidentFilter>) -> FieldResult<Vec<Incident>> {
    let filter = filter.unwrap_or_default();
    let mut vec: Vec<Incident> = context
        .incident_repo
        .find(&RepoIncidentFilter {
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
