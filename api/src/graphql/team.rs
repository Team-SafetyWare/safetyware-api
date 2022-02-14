use crate::crockford;
use crate::graphql::company::Company;
use crate::graphql::Context;
use crate::graphql::Person;
use crate::repo::team;
use derive_more::From;
use futures_util::StreamExt;
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From)]
pub struct Team(pub team::Team);

#[derive(juniper::GraphQLInputObject)]
pub struct TeamInput {
    pub name: String,
    pub company_id: String,
}

#[juniper::graphql_object(context = Context)]
impl Team {
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

    pub async fn people(&self, context: &Context) -> FieldResult<Vec<Person>> {
        Ok(context
            .team_repo
            .find_people(&self.0.id)
            .await?
            .map_err(anyhow::Error::from)
            .and_then(|tp| async move {
                context
                    .person_repo
                    .find_one(&tp.person_id)
                    .await
                    .map_err(Into::into)
            })
            .filter_map(|o| async move { o.transpose() })
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Team>> {
    Ok(context
        .team_repo
        .find_one(&id.to_string())
        .await?
        .map(Into::into))
}

pub async fn list(context: &Context) -> FieldResult<Vec<Team>> {
    Ok(context
        .team_repo
        .find(&Default::default())
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}

pub async fn create(context: &Context, input: TeamInput) -> FieldResult<Team> {
    let item = team::Team {
        id: crockford::random_id(),
        name: input.name,
        company_id: input.company_id,
    };
    context.team_repo.insert_one(&item).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .team_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}

pub async fn add_person(
    context: &Context,
    team_id: ID,
    person_id: ID,
) -> FieldResult<Option<Team>> {
    context.team_repo.add_person(&*team_id, &*person_id).await?;
    Ok(get(context, team_id).await?)
}

pub async fn remove_person(
    context: &Context,
    team_id: ID,
    person_id: ID,
) -> FieldResult<Option<Team>> {
    context
        .team_repo
        .remove_person(&*team_id, &*person_id)
        .await?;
    Ok(get(context, team_id).await?)
}
