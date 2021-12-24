use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::repo::DeleteResult;
use crate::v1::ResourceApi;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, IntoInfallible};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Reply};

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

#[derive(Clone)]
pub struct CompanyApi {
    pub repo: Arc<dyn CompanyRepo + Send + Sync + 'static>,
}

impl ResourceApi for CompanyApi {
    fn collection_name(&self) -> String {
        "companies".to_string()
    }

    fn get(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::get())
            .and(warp::path::param())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |id: String, s: Self| async move {
                let oid = id.parse().unwrap();
                let found = s.repo.find_one(oid).await.unwrap().map(Company::from);
                if let Some(company) = found {
                    let reply = company.as_json_reply();
                    Box::new(reply) as Box<dyn Reply>
                } else {
                    let reply = StatusCode::NOT_FOUND;
                    Box::new(reply) as Box<dyn Reply>
                }
                .into_infallible()
            })
            .boxed()
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::get())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |s: Self| async move {
                let companies: Vec<Company> = s
                    .repo
                    .find()
                    .await
                    .unwrap()
                    .map_ok(Into::into)
                    .try_collect()
                    .await
                    .unwrap();
                let reply = Box::new(companies.as_json_reply()) as Box<dyn Reply>;
                reply.into_infallible()
            })
            .boxed()
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        #[derive(Debug, Deserialize, Serialize)]
        struct Req {
            name: String,
        }
        warp::path(self.collection_name())
            .and(warp::post())
            .and(warp::body::json())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |req: Req, s: Self| async move {
                let company = RepoCompany {
                    id: Default::default(),
                    name: req.name,
                };
                s.repo.insert_one(&company).await.unwrap();
                let reply = Box::new(Company::from(company).as_json_reply()) as Box<dyn Reply>;
                reply.into_infallible()
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        warp::path(self.collection_name())
            .and(warp::delete())
            .and(warp::path::param())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |id: String, s: Self| async move {
                let oid = id.parse().unwrap();
                let res = s.repo.delete_one(oid).await.unwrap();
                let reply = match res {
                    DeleteResult::Deleted => Box::new(warp::reply()) as Box<dyn Reply>,
                    DeleteResult::NotFound => Box::new(StatusCode::NOT_FOUND) as Box<dyn Reply>,
                };
                reply.into_infallible()
            })
            .boxed()
    }

    fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        #[derive(Debug, Deserialize, Serialize)]
        struct Req {
            name: String,
        }
        warp::path(self.collection_name())
            .and(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |id: String, req: Req, s: Self| async move {
                let company = RepoCompany {
                    id: id.parse().unwrap(),
                    name: req.name,
                };
                s.repo.replace_one(&company).await.unwrap();
                let reply = Box::new(Company::from(company).as_json_reply()) as Box<dyn Reply>;
                reply.into_infallible()
            })
            .boxed()
    }
}
