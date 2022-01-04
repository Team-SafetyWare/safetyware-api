use crate::crockford;
use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::company;
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From)]
pub struct Company(company::Company);

#[derive(juniper::GraphQLInputObject)]
pub struct CompanyInput {
    name: String,
}

#[juniper::graphql_object(context = Context)]
impl Company {
    pub fn id(&self) -> ID {
        self.0.id.clone().into()
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub async fn people(&self, context: &Context) -> FieldResult<Vec<Person>> {
        // Todo: This is terribly inefficient.
        Ok(context
            .person_repo
            .find()
            .await?
            .try_filter_map(|p| async move {
                Ok(Some(p)
                    .filter(|p| p.company_id == self.0.id)
                    .map(Into::into))
            })
            .try_collect()
            .await?)
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Company>> {
    Ok(context
        .company_repo
        .find_one(&id.to_string())
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<Company>> {
    Ok(context
        .company_repo
        .find()
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: CompanyInput) -> FieldResult<Company> {
    let item = company::Company {
        id: crockford::random_id(),
        name: input.name,
    };
    context.company_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, id: ID, input: CompanyInput) -> FieldResult<Company> {
    let item = company::Company {
        id: id.to_string(),
        name: input.name,
    };
    context.company_repo.replace_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .company_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}