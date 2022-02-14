use crate::crockford;
use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::incident;
use crate::repo::incident::IncidentFilter as RepoIncidentFilter;
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From, Deref, DerefMut)]
pub struct Incident(pub incident::Incident);

#[derive(juniper::GraphQLInputObject)]
pub struct IncidentInput {
    pub timestamp: DateTime<Utc>,
    pub person_id: ID,
    pub coordinates: Vec<f64>,
    pub r#type: String,
}

#[derive(juniper::GraphQLInputObject, Default)]
pub struct IncidentFilter {
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[juniper::graphql_object(context = Context)]
impl Incident {
    pub fn id(&self) -> ID {
        self.id.clone().into()
    }

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

    pub fn r#type(&self) -> &str {
        &self.r#type
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Incident>> {
    Ok(context.incident_repo.find_one(&*id).await?.map(Into::into))
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
    vec.sort_by_key(|l| l.timestamp);
    Ok(vec)
}

pub async fn create(context: &Context, input: IncidentInput) -> FieldResult<Incident> {
    let item = incident::Incident {
        id: crockford::random_id(),
        timestamp: input.timestamp,
        person_id: input.person_id.to_string(),
        coordinates: input.coordinates,
        r#type: input.r#type,
    };
    context.incident_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, id: ID, input: IncidentInput) -> FieldResult<Incident> {
    let item = incident::Incident {
        id: id.to_string(),
        timestamp: input.timestamp,
        person_id: input.person_id.to_string(),
        coordinates: input.coordinates,
        r#type: input.r#type,
    };
    context.incident_repo.replace_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .incident_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
