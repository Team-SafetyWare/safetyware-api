use crate::graphql::Context;
use crate::repo::incident_stats;
use crate::repo::incident_stats::IncidentStatsFilter as RepoIncidentStatsFilter;
use chrono::{DateTime, Utc};
use derive_more::From;
use futures_util::TryStreamExt;
use juniper::FieldResult;

#[derive(Clone, From)]
pub struct IncidentStats(pub incident_stats::IncidentStats);

#[derive(juniper::GraphQLInputObject, Default)]
pub struct IncidentStatsFilter {
    pub person_ids: Option<Vec<String>>,
    pub min_timestamp: Option<DateTime<Utc>>,
    pub max_timestamp: Option<DateTime<Utc>>,
}

#[juniper::graphql_object(context = Context)]
impl IncidentStats {
    pub fn r#type(&self) -> &str {
        &self.0.r#type
    }

    pub fn count(&self) -> i32 {
        self.0.count
    }
}

pub async fn list(
    context: &Context,
    filter: Option<IncidentStatsFilter>,
) -> FieldResult<Vec<IncidentStats>> {
    let filter = filter.unwrap_or_default();
    Ok(context
        .incident_stats_repo
        .find(&RepoIncidentStatsFilter {
            person_ids: None,
            min_timestamp: filter.min_timestamp,
            max_timestamp: filter.max_timestamp,
        })
        .await?
        .map_ok(Into::into)
        .try_collect()
        .await?)
}
