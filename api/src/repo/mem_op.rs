use crate::common::{GetId, HasId};
use crate::repo::{DeleteError, DeleteResult, ItemStream, ReplaceError, ReplaceResult};
use futures_util::stream;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

pub type Collection<T> = Arc<RwLock<HashMap<<T as HasId>::Id, T>>>;

pub fn insert_one<T>(item: &T, collection: &Collection<T>) -> anyhow::Result<()>
where
    T: GetId + Clone + Sync,
    T::Id: Eq + Hash + Clone + Sync,
{
    let mut collection = collection.write().unwrap();
    if collection.contains_key(item.id()) {
        Err(anyhow::anyhow!("item already exists"))
    } else {
        collection.insert(item.id().clone(), item.clone());
        Ok(())
    }
}

pub fn replace_one<T>(item: &T, collection: &Collection<T>) -> ReplaceResult
where
    T: GetId + Clone,
    T::Id: Eq + Hash + Clone,
{
    let mut collection = collection.write().unwrap();
    if !collection.contains_key(item.id()) {
        Err(ReplaceError::NotFound)
    } else {
        collection.insert(item.id().clone(), item.clone());
        Ok(())
    }
}

pub fn find_one<T>(id: &T::Id, collection: &Collection<T>) -> anyhow::Result<Option<T>>
where
    T: HasId + Clone,
    T::Id: Eq + Hash + Clone,
{
    let collection = collection.read().unwrap();
    let found = collection.get(id).map(|item| item.clone());
    Ok(found)
}

pub fn find<T>(collection: &Collection<T>) -> anyhow::Result<Box<dyn ItemStream<T>>>
where
    T: HasId + Clone + Unpin + Send + 'static,
    T::Id: Eq + Hash + Clone + Unpin + Send,
{
    let collection = collection.read().unwrap();
    let values = collection.values().map(|i| i.clone());
    let results: Vec<anyhow::Result<T>> = values.map(|i| Ok(i)).collect();
    let stream = stream::iter(results);
    Ok(Box::new(stream))
}

pub fn delete_one<T>(id: &T::Id, collection: &Collection<T>) -> DeleteResult
where
    T: HasId + Clone,
    T::Id: Eq + Hash + Clone,
{
    let mut collection = collection.write().unwrap();
    if !collection.contains_key(id) {
        Err(DeleteError::NotFound)
    } else {
        collection.remove(id);
        Ok(())
    }
}

//noinspection DuplicatedCode
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{NewId, SetId};
    use crate::crockford;
    use futures_util::TryStreamExt;
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
            insert_one(&item, &collection).unwrap();

            // Assert.
            let opt = find_one(&item.id, &collection).unwrap();
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
            insert_one(&item, &collection).unwrap();

            // Act.
            let res = insert_one(&item, &collection);

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
            insert_one(&first, &collection).unwrap();
            let second = Item {
                id: id.clone(),
                name: crockford::random_id(),
            };

            // Act.
            replace_one(&second, &collection).unwrap();

            // Assert.
            let opt = find_one(&id, &collection).unwrap();
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
            insert_one(&item, &collection).unwrap();

            // Act.
            replace_one(&item, &collection).unwrap();

            // Assert.
            let opt = find_one(&item.id, &collection).unwrap();
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
            let res = replace_one(&item, &collection);

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
            insert_one(&item, &collection).unwrap();

            // Act.
            let opt = find_one(&item.id, &collection).unwrap();

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
            let opt = find_one(&id, &collection).unwrap();

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
                insert_one(item, &collection).unwrap();
            }

            // Act.
            let stream = find(&collection).unwrap();

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
            insert_one(&item, &collection).unwrap();

            // Act.
            delete_one(&item.id, &collection).unwrap();

            // Assert.
            let opt = find_one(&item.id, &collection).unwrap();
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
            let res = delete_one(&id, &collection);

            // Assert.
            assert!(matches!(res, Err(DeleteError::NotFound)));
        })
        .await
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Item {
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
        let collection: Collection<Item> = Default::default();
        test(collection).await;
    }
}
