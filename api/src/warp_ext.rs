use serde::Serialize;
use std::convert::Infallible;
use warp::{Filter, Reply};

pub fn with_clone<T: Clone + Send>(
    item: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || item.clone())
}

pub trait IntoInfallible {
    fn into_infallible(self) -> Result<Self, Infallible>
    where
        Self: Sized;
}

impl<T: Reply> IntoInfallible for T {
    fn into_infallible(self) -> Result<Self, Infallible> {
        Result::<_, Infallible>::Ok(self)
    }
}

pub trait AsJsonReply {
    fn as_json_reply(&self) -> warp::reply::Json;
}

impl<T: Serialize> AsJsonReply for T {
    fn as_json_reply(&self) -> warp::reply::Json {
        warp::reply::json(self)
    }
}
