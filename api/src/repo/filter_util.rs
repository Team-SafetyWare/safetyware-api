use bson::{Bson, Document};
use chrono::{DateTime, Utc};

pub trait InsertOpt {
    fn insert_opt<KT: Into<String>, BT: Into<Bson>>(
        &mut self,
        key: KT,
        val: Option<BT>,
    ) -> Option<Bson>;
}

impl InsertOpt for Document {
    fn insert_opt<KT: Into<String>, BT: Into<Bson>>(
        &mut self,
        key: KT,
        val: Option<BT>,
    ) -> Option<Bson> {
        match val {
            None => None,
            Some(val) => self.insert(key, val),
        }
    }
}

pub fn clamp_time(
    min_timestamp: Option<DateTime<Utc>>,
    max_timestamp: Option<DateTime<Utc>>,
) -> Option<Bson> {
    let mut doc: Option<Document> = None;
    if let Some(min_timestamp) = min_timestamp {
        doc.get_or_insert_with(Default::default)
            .insert("$gte", min_timestamp);
    }
    if let Some(max_timestamp) = max_timestamp {
        doc.get_or_insert_with(Default::default)
            .insert("$lt", max_timestamp);
    }
    doc.map(Into::into)
}

pub fn people(person_ids: Option<Vec<String>>) -> Bson {
    let mut doc = Document::new();
    if let Some(person_ids) = person_ids {
        doc.insert("$in", person_ids);
    }
    doc.into()
}
