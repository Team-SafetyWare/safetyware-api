use crate::crockford;
use crate::graphql::person::Person;
use crate::graphql::Context;
use crate::repo::company;
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::ID;

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

    pub async fn people(&self, context: &Context) -> Vec<Person> {
        context
            .person_repo
            .find()
            .await
            .unwrap()
            .try_filter_map(|p| async move {
                Ok(Some(p)
                    .filter(|p| p.company_id == self.0.id)
                    .map(Into::into))
            })
            .try_collect()
            .await
            .unwrap()
    }
}

pub async fn company(context: &Context, id: ID) -> Option<Company> {
    context
        .company_repo
        .find_one(&id.to_string())
        .await
        .unwrap()
        .map(Into::into)
}

pub async fn companies(context: &Context) -> Vec<Company> {
    context
        .company_repo
        .find()
        .await
        .unwrap()
        .map_ok(Into::into)
        .try_collect()
        .await
        .unwrap()
}

pub async fn create_company(context: &Context, input: CompanyInput) -> Company {
    let item = company::Company {
        id: crockford::random_id(),
        name: input.name,
    };
    context.company_repo.insert_one(&item).await.unwrap();
    item.into()
}

pub async fn replace_company(context: &Context, id: ID, input: CompanyInput) -> Company {
    let item = company::Company {
        id: id.to_string(),
        name: input.name,
    };
    context.company_repo.replace_one(&item).await.unwrap();
    item.into()
}

pub async fn delete_company(context: &Context, id: ID) -> ID {
    context
        .company_repo
        .delete_one(&id.clone().to_string())
        .await
        .unwrap();
    id
}
