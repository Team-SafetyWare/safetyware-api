use crate::repo::company::CompanyRepo;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, IntoInfallible};
use futures_util::TryStreamExt;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub const PREFIX: &str = "companies";

pub fn filter<R: CompanyRepo>(repo: R) -> BoxedFilter<(impl Reply,)> {
    warp::path(PREFIX).and(list(repo.clone())).boxed()
}

fn list<R: CompanyRepo>(repo: R) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp_ext::with_clone(repo))
        .and_then(move |repo: R| async move {
            // Todo: Do not unwrap.
            let companies: Vec<_> = repo.find().await.unwrap().try_collect().await.unwrap();
            companies.as_json_reply().into_infallible()
        })
        .boxed()
}
