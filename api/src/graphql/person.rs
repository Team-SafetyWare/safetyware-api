use crate::crockford;
use crate::graphql::company::Company;
use crate::graphql::location_reading::LocationReading;
use crate::graphql::Context;
use crate::repo::person;
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From)]
pub struct Person(person::Person);

#[derive(juniper::GraphQLInputObject)]
pub struct PersonInput {
    name: String,
    company_id: String,
}

#[juniper::graphql_object(context = Context)]
impl Person {
    pub fn id(&self) -> ID {
        self.0.id.clone().into()
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub async fn company(&self, context: &Context) -> FieldResult<Option<Company>> {
        Ok(context
            .company_repo
            .find_one(&self.0.company_id)
            .await?
            .map(Into::into))
    }

    pub async fn location_readings(&self, context: &Context) -> FieldResult<Vec<LocationReading>> {
        // Todo: This is terribly inefficient.
        Ok(context
            .location_reading_repo
            .find()
            .await?
            .try_filter_map(|lr| async move {
                Ok(Some(lr)
                    .filter(|lr| lr.person_id == self.0.id)
                    .map(Into::into))
            })
            .try_collect()
            .await?)
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Person>> {
    Ok(context
        .person_repo
        .find_one(&id.to_string())
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<Person>> {
    Ok(context
        .person_repo
        .find()
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: PersonInput) -> FieldResult<Person> {
    let item = person::Person {
        id: crockford::random_id(),
        name: input.name,
        company_id: input.company_id,
    };
    context.person_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, id: ID, input: PersonInput) -> FieldResult<Person> {
    let item = person::Person {
        id: id.to_string(),
        name: input.name,
        company_id: input.company_id,
    };
    context.person_repo.replace_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .person_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
