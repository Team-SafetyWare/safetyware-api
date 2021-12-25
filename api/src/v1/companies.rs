use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::repo::DeleteResult;
use crate::v1::ResourceApi;
use crate::warp_ext;
use crate::warp_ext::{AsJsonReply, IntoInfallible};
use anyhow::Context;
use bson::oid::ObjectId;
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Reply};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    #[serde(skip_deserializing)]
    pub id: Option<String>,
    pub name: String,
}

impl From<RepoCompany> for Company {
    fn from(value: RepoCompany) -> Self {
        Self {
            id: Some(value.id.to_string()),
            name: value.name,
        }
    }
}

impl TryFrom<Company> for RepoCompany {
    type Error = anyhow::Error;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.context("id missing")?.parse()?,
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
        warp::path(self.collection_name())
            .and(warp::post())
            .and(warp::body::json())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(move |mut company: Company, s: Self| async move {
                company.id = Some(ObjectId::new().to_string());
                s.repo
                    .insert_one(&company.clone().try_into().unwrap())
                    .await
                    .unwrap();
                let reply = Box::new(company.as_json_reply()) as Box<dyn Reply>;
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
        warp::path(self.collection_name())
            .and(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and(warp_ext::with_clone(self.clone()))
            .and_then(
                move |id: String, mut company: Company, s: Self| async move {
                    company.id = Some(id.parse().unwrap());
                    s.repo
                        .replace_one(&company.clone().try_into().unwrap())
                        .await
                        .unwrap();
                    let reply = Box::new(company.as_json_reply()) as Box<dyn Reply>;
                    reply.into_infallible()
                },
            )
            .boxed()
    }
}
