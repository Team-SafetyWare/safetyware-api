use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, IntoInfallible};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

pub const PREFIX: &str = "companies";

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
}

impl From<RepoCompany> for Company {
    fn from(value: RepoCompany) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
        }
    }
}

impl TryFrom<Company> for RepoCompany {
    type Error = anyhow::Error;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.name.parse()?,
            name: value.name,
        })
    }
}

pub fn filter(repo: impl CompanyRepo) -> BoxedFilter<(impl Reply,)> {
    warp::path(PREFIX).and(list(repo.clone())).boxed()
}

fn list<R: CompanyRepo>(repo: R) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp_ext::with_clone(repo))
        .and_then(move |repo: R| async move {
            let companies: Vec<Company> = repo
                .find()
                .await
                .unwrap()
                .map_ok(Into::into)
                .try_collect()
                .await
                .unwrap();
            companies.as_json_reply().into_infallible()
        })
        .boxed()
}
