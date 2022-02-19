use bson::{Bson, Document};
use chrono::{DateTime, Utc};

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

pub fn people(person_ids: Option<Vec<String>>) -> Bson {
    let mut doc = Document::new();
    if let Some(person_ids) = person_ids {
        doc.insert("$in", person_ids);
    }
    doc.into()
}
