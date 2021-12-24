use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, IntoInfallible};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
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
    let filters = get(repo.clone())
        .or(list(repo.clone()))
        .or(create(repo.clone()))
        .or(update(repo.clone()))
        .or(delete(repo.clone()))
        .or(replace(repo.clone()));
    warp::path(PREFIX).and(filters).boxed()
}

fn get<R: CompanyRepo>(repo: R) -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::path::param())
        .and(warp_ext::with_clone(repo))
        .and_then(move |id: String, repo: R| async move {
            let oid = id.parse().unwrap();
            let found = repo.find_one(oid).await.unwrap();
            if let Some(company) = found {
                let reply = company.as_json_reply();
                Box::new(reply) as Box<dyn warp::Reply>
            } else {
                let reply = warp::reply::with_status(warp::reply(), StatusCode::NOT_FOUND);
                Box::new(reply) as Box<dyn warp::Reply>
            }
            .into_infallible()
        })
        .boxed()
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

fn create<R: CompanyRepo>(repo: R) -> BoxedFilter<(impl Reply,)> {
    #[derive(Debug, Deserialize, Serialize)]
    struct Req {
        name: String,
    }
    warp::post()
        .and(warp::body::json())
        .and(warp_ext::with_clone(repo))
        .and_then(move |req: Req, repo: R| async move {
            let company = RepoCompany {
                id: Default::default(),
                name: req.name,
            };
            repo.insert_one(&company).await.unwrap();
            warp::reply().into_infallible()
        })
        .boxed()
}

fn update<R: CompanyRepo>(_: R) -> BoxedFilter<(impl Reply,)> {
    warp::patch()
        .and_then(move || async move {
            todo!();
            #[allow(unreachable_code)]
            warp::reply().into_infallible()
        })
        .boxed()
}

fn delete<R: CompanyRepo>(_: R) -> BoxedFilter<(impl Reply,)> {
    warp::delete()
        .and_then(move || async move {
            todo!();
            #[allow(unreachable_code)]
            warp::reply().into_infallible()
        })
        .boxed()
}

fn replace<R: CompanyRepo>(_: R) -> BoxedFilter<(impl Reply,)> {
    warp::put()
        .and_then(move || async move {
            todo!();
            #[allow(unreachable_code)]
            warp::reply().into_infallible()
        })
        .boxed()
}
