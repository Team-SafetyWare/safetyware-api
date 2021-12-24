use bson::Bson;
use futures_util::Stream;
use futures_util::TryStreamExt;
use mongodb::Collection;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod company;
mod mongo_common;

pub trait ItemStream<T>: Stream<Item = anyhow::Result<T>> {}

impl<T, I> ItemStream<I> for T where T: Stream<Item = anyhow::Result<I>> {}
