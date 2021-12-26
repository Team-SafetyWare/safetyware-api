use crate::repo;
use api::common::{GetId, HasId, NewId, SetId};
use api::repo::mongo_op;
use bson::oid::ObjectId;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::future::Future;
use uuid::Uuid;

#[tokio::test]
async fn test_insert_one() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };

        // Act.
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(item.id, &collection).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found.id, item.id);
        assert_eq!(found.name, item.name);
    })
    .await
}

#[tokio::test]
async fn test_replace_one() {
    test_op(|collection| async move {
        // Arrange.
        let id = Default::default();
        let first = Item {
            id,
            name: Uuid::new_v4().to_string(),
        };
        mongo_op::insert_one(&first, &collection).await.unwrap();
        let second = Item {
            id,
            name: Uuid::new_v4().to_string(),
        };

        // Act.
        mongo_op::replace_one(&second, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(id, &collection).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found.name, second.name);
    })
    .await
}

#[tokio::test]
async fn test_find_one() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Act.
        let opt = mongo_op::find_one(item.id, &collection).await.unwrap();

        // Assert.
        let found = opt.expect("not found");
        assert_eq!(found.id, item.id);
        assert_eq!(found.name, item.name);
    })
    .await
}

#[tokio::test]
async fn test_find() {
    test_op(|collection| async move {
        // Arrange.
        let items: Vec<_> = (0..3)
            .map(|_| Item {
                id: Default::default(),
                name: Uuid::new_v4().to_string(),
            })
            .collect();
        for item in &items {
            mongo_op::insert_one(item, &collection).await.unwrap();
        }

        // Act.
        let stream = mongo_op::find(&collection).await.unwrap();

        // Assert.
        let found: Vec<_> = stream.try_collect().await.unwrap();
        assert_eq!(found.len(), items.len());
        for item in items {
            assert!(found.iter().any(|c| c.id == item.id))
        }
    })
    .await
}

#[tokio::test]
async fn test_delete_one() {
    test_op(|collection| async move {
        let item = Item {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        mongo_op::delete_one(item.id, &collection).await.unwrap();

        let opt = mongo_op::find_one(item.id, &collection).await.unwrap();
        assert!(opt.is_none());
    })
    .await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

impl HasId for Item {
    type Id = ObjectId;
}

impl GetId for Item {
    fn id(&self) -> Self::Id {
        self.id
    }
}

impl SetId for Item {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

impl NewId for Item {
    fn new_id() -> Self::Id {
        Default::default()
    }
}

async fn test_op<T, F>(test: T)
where
    T: Fn(Collection<Item>) -> F,
    F: Future,
{
    let db = repo::new_db().await.unwrap();
    let collection = db.collection("item");
    test(collection).await;
    db.drop(None).await.unwrap();
}
