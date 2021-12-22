use bson::oid::ObjectId;
use mongodb::options::ReplaceOptions;
use mongodb::{Collection, Cursor, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

impl Company {
    pub fn collection(db: &Database) -> Collection<Company> {
        db.collection("company")
    }

    // Todo: Disallow dead code once used.
    #[allow(dead_code)]
    pub async fn find_one(id: ObjectId, db: &Database) -> anyhow::Result<Option<Company>> {
        let coll = Self::collection(db);
        let filter = bson::doc! {"_id": id};
        let opt = coll.find_one(filter, None).await?;
        Ok(opt)
    }

    // Todo: Disallow dead code once used.
    #[allow(dead_code)]
    pub async fn list(db: &Database) -> anyhow::Result<Cursor<Company>> {
        let coll = Self::collection(db);
        let cursor = coll.find(None, None).await?;
        Ok(cursor)
    }

    // Todo: Disallow dead code once used.
    #[allow(dead_code)]
    pub async fn upsert(&self, db: &Database) -> anyhow::Result<()> {
        let coll = Self::collection(db);
        let query = bson::doc! {"_id": self.id};
        let options = ReplaceOptions::builder().upsert(true).build();
        coll.replace_one(query, self, options).await?;
        Ok(())
    }

    // Todo: Disallow dead code once used.
    #[allow(dead_code)]
    pub async fn delete(&self, db: &Database) -> anyhow::Result<()> {
        let coll = Self::collection(db);
        let query = bson::doc! {"_id": self.id};
        coll.delete_one(query, None).await?;
        Ok(())
    }
}
