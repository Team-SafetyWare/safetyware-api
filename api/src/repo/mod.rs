use futures_util::Stream;

pub mod company;
pub mod location_reading;
pub mod person;
pub mod user_account;

pub trait ItemStream<T: Unpin + Send>: Stream<Item = anyhow::Result<T>> + Unpin + Send {}

impl<T, I> ItemStream<I> for T
where
    T: Stream<Item = anyhow::Result<I>> + Unpin + Send,
    I: Unpin + Send,
{
}

#[derive(thiserror::Error, Debug)]
pub enum ReplaceError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type ReplaceResult = Result<(), ReplaceError>;

#[derive(thiserror::Error, Debug)]
pub enum DeleteError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type DeleteResult = Result<(), DeleteError>;
