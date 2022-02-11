use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::device;
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From)]
pub struct Device(pub device::Device);

#[derive(juniper::GraphQLInputObject)]
pub struct DeviceInput {
    pub id: ID,
    pub owner_id: ID,
}

#[juniper::graphql_object(context = Context)]
impl Device {
    pub fn id(&self) -> ID {
        self.0.id.clone().into()
    }

    pub async fn owner(&self, context: &Context) -> FieldResult<Option<Person>> {
        Ok(context
            .person_repo
            .find_one(&self.0.owner_id)
            .await?
            .map(Into::into))
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Device>> {
    Ok(context
        .device_repo
        .find_one(&id.to_string())
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<Device>> {
    Ok(context
        .device_repo
        .find(&Default::default())
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: DeviceInput) -> FieldResult<Device> {
    let item = device::Device {
        id: input.id.to_string(),
        owner_id: input.owner_id.to_string(),
    };
    context.device_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, input: DeviceInput) -> FieldResult<Device> {
    let item = device::Device {
        id: input.id.to_string(),
        owner_id: input.owner_id.to_string(),
    };
    context.device_repo.replace_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .device_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
