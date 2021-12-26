use serde::Serialize;
use std::convert::Infallible;
use warp::{Filter, Reply};

pub fn with_clone<T: Clone + Send>(
    item: T,
) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || item.clone())
}

pub trait AsJsonReply {
    fn as_json_reply(&self) -> warp::reply::Json;
}

impl<T: Serialize> AsJsonReply for T {
    fn as_json_reply(&self) -> warp::reply::Json {
        warp::reply::json(self)
    }
}

pub trait BoxReply {
    fn boxed(self) -> Box<dyn Reply>;
}

impl<T: Reply + 'static> BoxReply for T {
    fn boxed(self) -> Box<dyn Reply> {
        Box::new(self) as Box<dyn Reply>
    }
}
