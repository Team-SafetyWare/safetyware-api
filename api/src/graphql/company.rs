use crate::crockford;
use crate::graphql::incident_stats::{IncidentStats, IncidentStatsFilter};
use crate::graphql::person::Person;
use crate::graphql::team::Team;
use crate::graphql::user_account::UserAccount;
use crate::graphql::Context;
use crate::repo::company;
use crate::repo::incident_stats::IncidentStatsFilter as RepoIncidentStatsFilter;
use crate::repo::person::PersonFilter;
use crate::repo::team::TeamFilter;
use crate::repo::user_account::UserAccountFilter;
use derive_more::{Deref, DerefMut, From};
use futures_util::TryStreamExt;
use juniper::{FieldResult, ID};

#[derive(Clone, From, Deref, DerefMut)]
pub struct Company(pub company::Company);

#[derive(juniper::GraphQLInputObject)]
pub struct CompanyInput {
    pub name: String,
}

#[juniper::graphql_object(context = Context)]
impl Company {
    pub fn id(&self) -> ID {
        self.id.clone().into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn incident_stats(
        &self,
        context: &Context,
        filter: Option<IncidentStatsFilter>,
    ) -> FieldResult<Vec<IncidentStats>> {
        let filter = filter.unwrap_or_default();
        let person_ids = context
            .person_repo
            .find(PersonFilter {
                company_ids: Some(vec![self.id.clone()]),
            })
            .await?
            .map_ok(|p| p.id)
            .try_collect()
            .await?;
        Ok(context
            .incident_stats_repo
            .find(RepoIncidentStatsFilter {
                person_ids: Some(person_ids),
                min_timestamp: filter.min_timestamp,
                max_timestamp: filter.max_timestamp,
            })
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }

    pub async fn people(&self, context: &Context) -> FieldResult<Vec<Person>> {
        Ok(context
            .person_repo
            .find(PersonFilter {
                company_ids: Some(vec![self.id.clone()]),
            })
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }

    pub async fn teams(&self, context: &Context) -> FieldResult<Vec<Team>> {
        Ok(context
            .team_repo
            .find(TeamFilter {
                company_ids: Some(vec![self.id.clone()]),
            })
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }

    pub async fn user_accounts(&self, context: &Context) -> FieldResult<Vec<UserAccount>> {
        Ok(context
            .user_account_repo
            .find(UserAccountFilter {
                company_ids: Some(vec![self.id.clone()]),
            })
            .await?
            .map_ok(Into::into)
            .try_collect()
            .await?)
    }
}

pub async fn get(context: &Context, id: ID) -> FieldResult<Option<Company>> {
    Ok(context.company_repo.find_one(&*id).await?.map(Into::into))
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
    context.company_repo.insert_one(item.clone()).await?;
    Ok(item.into())
}

pub async fn replace(context: &Context, id: ID, input: CompanyInput) -> FieldResult<Company> {
    let item = company::Company {
        id: id.to_string(),
        name: input.name,
    };
    context.company_repo.replace_one(item.clone()).await?;
    Ok(item.into())
}

pub async fn delete(context: &Context, id: ID) -> FieldResult<ID> {
    context
        .company_repo
        .delete_one(&id.clone().to_string())
        .await?;
    Ok(id)
}
