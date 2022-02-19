use crate::repo::team::TeamRepo;
use bson::{Bson, Document};
use chrono::{DateTime, Utc};
use futures_util::{stream, StreamExt, TryFutureExt, TryStreamExt};
use std::collections::HashSet;

const MAX_CONCURRENT_DB_CALLS: usize = 10;

pub fn clamp_timestamp(
    min_timestamp: Option<DateTime<Utc>>,
    max_timestamp: Option<DateTime<Utc>>,
) -> Bson {
    let mut doc = Document::new();
    if let Some(min_timestamp) = min_timestamp {
        doc.insert("$gte", min_timestamp);
    }
    if let Some(max_timestamp) = max_timestamp {
        doc.insert("$lt", max_timestamp);
    }
    doc.into()
}

pub async fn person_or_team(
    person_ids: Option<Vec<String>>,
    team_ids: Option<Vec<String>>,
    team_repo: &dyn TeamRepo + Send + Sync,
) -> anyhow::Result<Bson> {
    let person_ids = person_ids.unwrap_or_default();
    let team_ids = team_ids.unwrap_or_default();
    let combined_person_ids: Vec<String> = stream::iter(team_ids)
        .map(|team_id| async move { team_repo.find_people(&team_id).await })
        .buffered(MAX_CONCURRENT_DB_CALLS)
        .try_flatten()
        .map_ok(|tp| tp.person_id)
        .chain(stream::iter(person_ids.into_iter().map(Ok)))
        .try_collect::<HashSet<String>>()
        .await?
        .into_iter()
        .collect();
    Ok((bson::doc! { "$in": combined_person_ids }).into())
}
