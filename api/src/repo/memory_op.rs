use crate::common::{GetId, HasId};
use crate::repo::{DeleteError, DeleteResult, ItemStream, ReplaceError, ReplaceResult};
use futures_util::stream;
use std::collections::HashMap;
use std::hash::Hash;

pub type Collection<T> = HashMap<<T as HasId>::Id, T>;

pub fn insert_one<T>(item: &T, collection: &mut Collection<T>) -> anyhow::Result<()>
where
    T: GetId + Clone,
    T::Id: Eq + Hash + Clone,
{
    if collection.contains_key(item.id()) {
        Err(anyhow::anyhow!("item already exists"))
    } else {
        collection.insert(item.id().clone(), item.clone());
        Ok(())
    }
}

pub fn replace_one<T>(item: &T, collection: &mut Collection<T>) -> ReplaceResult
where
    T: GetId + Clone,
    T::Id: Eq + Hash + Clone,
{
    if !collection.contains_key(item.id()) {
        Err(ReplaceError::NotFound)
    } else {
        collection.insert(item.id().clone(), item.clone());
        Ok(())
    }
}

pub fn find_one<T>(id: &T::Id, collection: &mut Collection<T>) -> anyhow::Result<Option<T>>
where
    T: HasId + Clone,
    T::Id: Eq + Hash + Clone,
{
    let found = collection.get(id).map(|item| item.clone());
    Ok(found)
}

pub fn find<T>(collection: &mut Collection<T>) -> anyhow::Result<Box<dyn ItemStream<T>>>
where
    T: HasId + Clone + Unpin + Send + 'static,
    T::Id: Eq + Hash + Clone + Unpin + Send,
{
    let values = collection.values().map(|i| i.clone());
    let results: Vec<anyhow::Result<T>> = values.map(|i| Ok(i)).collect();
    let stream = stream::iter(results);
    Ok(Box::new(stream))
}

pub fn delete_one<T>(id: &T::Id, collection: &mut Collection<T>) -> DeleteResult
where
    T: HasId + Clone,
    T::Id: Eq + Hash + Clone,
{
    if !collection.contains_key(id) {
        Err(DeleteError::NotFound)
    } else {
        collection.remove(id);
        Ok(())
    }
}
