use crate::crockford;
use crate::graphql::company::Company;
use crate::graphql::device::Device;
use crate::graphql::gas_reading::GasReading;
use crate::graphql::incident::{Incident, IncidentFilter};
use crate::graphql::location_reading::LocationReading;
use crate::graphql::Context;
use crate::graphql::GasReadingFilter;
use crate::graphql::LocationReadingFilter;
use crate::repo::device::DeviceFilter as RepoDeviceFilter;
use crate::repo::gas_reading::GasReadingFilter as RepoGasReadingFilter;
use crate::repo::incident::IncidentFilter as RepoIncidentFilter;
use crate::repo::location_reading::LocationReadingFilter as RepoLocationReadingFilter;
use crate::repo::person;
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From, Deref, DerefMut)]
pub struct Person(pub person::Person);

#[derive(juniper::GraphQLInputObject)]
pub struct PersonInput {
    pub name: String,
    pub company_id: ID,
}

#[juniper::graphql_object(context = Context)]
impl Person {
    pub fn id(&self) -> ID {
        self.id.clone().into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn company(&self, context: &Context) -> FieldResult<Option<Company>> {
        Ok(context
            .company_repo
            .find_one(&self.company_id)
            .await?
            .map(Into::into))
    }

    pub async fn devices(&self, context: &Context) -> FieldResult<Vec<Device>> {
        Ok(context
            .device_repo
            .find(RepoDeviceFilter {
                owner_ids: Some(vec![self.id.clone()]),
            })
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }

    pub async fn gas_readings(
        &self,
        context: &Context,
        filter: Option<GasReadingFilter>,
    ) -> FieldResult<Vec<GasReading>> {
        let filter = filter.unwrap_or_default();
        let mut vec: Vec<GasReading> = context
            .gas_reading_repo
            .find(RepoGasReadingFilter {
                person_ids: Some(vec![self.id.clone()]),
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

    pub async fn incidents(
        &self,
        context: &Context,
        filter: Option<IncidentFilter>,
    ) -> FieldResult<Vec<Incident>> {
        let filter = filter.unwrap_or_default();
        let mut vec: Vec<Incident> = context
            .incident_repo
            .find(RepoIncidentFilter {
                person_ids: Some(vec![self.id.clone()]),
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

    pub async fn location_readings(
        &self,
        context: &Context,
        filter: Option<LocationReadingFilter>,
    ) -> FieldResult<Vec<LocationReading>> {
        let filter = filter.unwrap_or_default();
        let mut vec: Vec<LocationReading> = context
            .location_reading_repo
            .find(RepoLocationReadingFilter {
                person_ids: Some(vec![self.id.clone()]),
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
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Person>> {
    Ok(context.person_repo.find_one(&*id).await?.map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<Person>> {
    Ok(context
        .person_repo
        .find(Default::default())
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: PersonInput) -> FieldResult<Person> {
    let item = person::Person {
        id: crockford::random_id(),
        name: input.name,
        company_id: input.company_id.to_string(),
    };
    context.person_repo.insert_one(item.clone()).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, id: ID, input: PersonInput) -> FieldResult<Person> {
    let item = person::Person {
        id: id.to_string(),
        name: input.name,
        company_id: input.company_id.to_string(),
    };
    context.person_repo.replace_one(item.clone()).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .person_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
