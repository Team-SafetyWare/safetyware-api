use crate::repo;
use api::common::{GetId, HasId, NewId, SetId};
use api::crockford;
use api::repo::{mongo_op, DeleteError, ReplaceError};
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use std::future::Future;

#[tokio::test]
async fn test_insert_one_new() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };

        // Act.
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(&item.id, &collection).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found, item);
    })
    .await
}

#[tokio::test]
async fn test_insert_one_existing() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Act.
        let res = mongo_op::insert_one(&item, &collection).await;

        // Assert.
        assert!(res.is_err());
    })
    .await
}

#[tokio::test]
async fn test_replace_one_modified() {
    test_op(|collection| async move {
        // Arrange.
        let id = Item::new_id();
        let first = Item {
            id: id.clone(),
            name: crockford::random_id(),
        };
        mongo_op::insert_one(&first, &collection).await.unwrap();
        let second = Item {
            id: id.clone(),
            name: crockford::random_id(),
        };

        // Act.
        mongo_op::replace_one(&second, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(&id, &collection).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found, second);
    })
    .await
}

#[tokio::test]
async fn test_replace_one_unmodified() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Act.
        mongo_op::replace_one(&item, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(&item.id, &collection).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found, item);
    })
    .await
}

#[tokio::test]
async fn test_replace_one_missing() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };

        // Act.
        let res = mongo_op::replace_one(&item, &collection).await;

        // Assert.
        assert!(matches!(res, Err(ReplaceError::NotFound)));
    })
    .await
}

#[tokio::test]
async fn test_find_one_exists() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Act.
        let opt = mongo_op::find_one(&item.id, &collection).await.unwrap();

        // Assert.
        let found = opt.expect("not found");
        assert_eq!(found, item);
    })
    .await
}

#[tokio::test]
async fn test_find_one_missing() {
    test_op(|collection| async move {
        // Arrange.
        let id = Item::new_id();

        // Act.
        let opt = mongo_op::find_one(&id, &collection).await.unwrap();

        // Assert.
        assert!(opt.is_none());
    })
    .await
}

#[tokio::test]
async fn test_find() {
    test_op(|collection| async move {
        // Arrange.
        let items: Vec<_> = (0..3)
            .map(|_| Item {
                id: Item::new_id(),
                name: crockford::random_id(),
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
            assert!(found.iter().any(|c| *c == item));
        }
    })
    .await
}

#[tokio::test]
async fn test_delete_one_existing() {
    test_op(|collection| async move {
        // Arrange.
        let item = Item {
            id: Item::new_id(),
            name: crockford::random_id(),
        };
        mongo_op::insert_one(&item, &collection).await.unwrap();

        // Act.
        mongo_op::delete_one(&item.id, &collection).await.unwrap();

        // Assert.
        let opt = mongo_op::find_one(&item.id, &collection).await.unwrap();
        assert!(opt.is_none());
    })
    .await
}

#[tokio::test]
async fn test_delete_one_missing() {
    test_op(|collection| async move {
        // Arrange.
        let id = Item::new_id();

        // Act.
        let res = mongo_op::delete_one(&id, &collection).await;

        // Assert.
        assert!(matches!(res, Err(DeleteError::NotFound)));
    })
    .await
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct Item {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

impl HasId for Item {
    type Id = String;
}

impl GetId for Item {
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl SetId for Item {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

impl NewId for Item {
    fn new_id() -> Self::Id {
        crockford::random_id()
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
