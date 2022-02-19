use bson::{Bson, Document};

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

pub mod filter {
    use bson::{Bson, Document};

    pub fn clamp<T: Into<Bson>>(
        min_inclusive: Option<T>,
        max_exclusive: Option<T>,
    ) -> Option<Bson> {
        let mut doc: Option<Document> = None;
        if let Some(min) = min_inclusive {
            doc.get_or_insert_with(Default::default).insert("$gte", min);
        }
        if let Some(max) = max_exclusive {
            doc.get_or_insert_with(Default::default).insert("$lt", max);
        }
        doc.map(Into::into)
    }

    pub fn one_of<T: Into<Bson>>(values: Option<Vec<T>>) -> Option<Bson> {
        match values {
            None => None,
            Some(values) => Some((bson::doc! { "$in":  values }).into()),
        }
    }
}
