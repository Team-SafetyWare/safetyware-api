use crate::crockford;
use crate::graphql::company::Company;
use crate::graphql::Context;
use crate::repo::user_account;
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From, Deref, DerefMut)]
pub struct UserAccount(pub user_account::UserAccount);

#[derive(juniper::GraphQLInputObject)]
pub struct UserAccountInput {
    pub name: String,
    pub title: String,
    pub email: String,
    pub phone: String,
    pub company_id: ID,
}

#[juniper::graphql_object(context = Context)]
impl UserAccount {
    pub fn id(&self) -> ID {
        self.id.clone().into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }

    pub async fn company(&self, context: &Context) -> FieldResult<Option<Company>> {
        Ok(context
            .company_repo
            .find_one(&self.company_id)
            .await?
            .map(Into::into))
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<UserAccount>> {
    Ok(context
        .user_account_repo
        .find_one(&*id)
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<UserAccount>> {
    Ok(context
        .user_account_repo
        .find(Default::default())
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: UserAccountInput) -> FieldResult<UserAccount> {
    let item = user_account::UserAccount {
        id: crockford::random_id(),
        name: input.name,
        title: input.title,
        email: input.email,
        phone: input.phone,
        company_id: input.company_id.to_string(),
    };
    context.user_account_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn replace(
    context: &Context,
    id: ID,
    input: UserAccountInput,
) -> FieldResult<UserAccount> {
    let item = user_account::UserAccount {
        id: id.to_string(),
        name: input.name,
        title: input.title,
        email: input.email,
        phone: input.phone,
        company_id: input.company_id.to_string(),
    };
    context.user_account_repo.replace_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .user_account_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
