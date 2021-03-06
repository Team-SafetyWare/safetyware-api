use crate::crockford;
use crate::graphql::company::Company;
use crate::graphql::Context;
use crate::image::PngBytes;
use crate::repo::user_account;
use anyhow::Context as AnyhowContext;
use data_encoding::BASE64;
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From, Deref, DerefMut)]
pub struct UserAccount(pub user_account::UserAccount);

#[derive(Debug, Copy, Clone, juniper::GraphQLEnum)]
pub enum Access {
    View,
    Admin,
}

impl From<Access> for user_account::Access {
    fn from(value: Access) -> Self {
        match value {
            Access::View => Self::View,
            Access::Admin => Self::Admin,
        }
    }
}

impl From<user_account::Access> for Access {
    fn from(value: user_account::Access) -> Self {
        match value {
            user_account::Access::View => Self::View,
            user_account::Access::Admin => Self::Admin,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct UserAccountInput {
    pub name: String,
    pub access: Access,
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

    pub fn access(&self) -> Access {
        self.access.into()
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
        access: input.access.into(),
        title: input.title,
        email: input.email,
        phone: input.phone,
        company_id: input.company_id.to_string(),
    };
    context.user_account_repo.insert_one(item.clone()).await?;
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
        access: input.access.into(),
        title: input.title,
        email: input.email,
        phone: input.phone,
        company_id: input.company_id.to_string(),
    };
    context.user_account_repo.replace_one(item.clone()).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .user_account_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}

pub async fn login(
    context: &Context,
    user_account_id: ID,
    password: String,
) -> FieldResult<String> {
    context
        .auth_provider
        .verify_password(&user_account_id, &password)
        .await?
        .map_err(|_| "Incorrect password")?;
    let user_account = context
        .user_account_repo
        .find_one(&user_account_id)
        .await?
        .context("User account not found")?;
    let token = context.claims_provider.create_token(&user_account)?;
    Ok(token)
}

pub async fn set_password(
    context: &Context,
    user_account_id: ID,
    password: String,
) -> FieldResult<bool> {
    context
        .auth_provider
        .set_password(&user_account_id, &password)
        .await?;
    Ok(true)
}

pub async fn set_profile_image(
    context: &Context,
    user_account_id: ID,
    image_base64: String,
) -> FieldResult<String> {
    let image_bytes = BASE64.decode(image_base64.as_bytes())?;
    let image = image::load_from_memory(&image_bytes)?;
    let png_bytes = image.png_bytes()?;
    context
        .user_account_repo
        .set_profile_image_png(&user_account_id, png_bytes)
        .await?;
    Ok(format!("/v1/userAccount/{}/profile.png", user_account_id))
}
