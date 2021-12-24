use futures_util::Stream;

pub mod company;
mod mongo_common;

pub trait ItemStream<T: Unpin + Send>: Stream<Item = anyhow::Result<T>> + Unpin + Send {}

impl<T, I> ItemStream<I> for T
where
    T: Stream<Item = anyhow::Result<I>> + Unpin + Send,
    I: Unpin + Send,
{
}
