use futures_util::Stream;

pub mod company;

pub trait ItemStream<T>: Stream<Item = anyhow::Result<T>> {}

impl<T, I> ItemStream<I> for T where T: Stream<Item = anyhow::Result<I>> {}
