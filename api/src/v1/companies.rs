use crate::common::{GetId, HasId, SetId};
use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::repo::DeleteResult;
use crate::v1::op;
use crate::v1::{ResourceApi, ResourceOperation};
use crate::warp_ext::{AsJsonReply, BoxReplyInfallible};
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

impl HasId for Company {
    type Id = Option<String>;
}

impl GetId for Company {
    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl SetId for Company {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
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
        op::get::<Company, RepoCompany, _>(self.repo.clone(), self.collection_name())
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        self.operation(warp::get())
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
                companies.as_json_reply().boxed_infallible()
            })
            .boxed()
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        self.operation(warp::post())
            .and(warp::body::json())
            .and_then(move |s: Self, mut company: Company| async move {
                company.id = Some(ObjectId::new().to_string());
                s.repo
                    .insert_one(&company.clone().try_into().unwrap())
                    .await
                    .unwrap();
                company.as_json_reply().boxed_infallible()
            })
            .boxed()
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        self.operation(warp::delete())
            .and(warp::path::param())
            .and_then(move |s: Self, id: String| async move {
                let oid = id.parse().unwrap();
                let res = s.repo.delete_one(oid).await.unwrap();
                match res {
                    DeleteResult::Deleted => warp::reply().boxed_infallible(),
                    DeleteResult::NotFound => StatusCode::NOT_FOUND.boxed_infallible(),
                }
            })
            .boxed()
    }

    fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        self.operation(warp::put())
            .and(warp::path::param())
            .and(warp::body::json())
            .and_then(
                move |s: Self, id: String, mut company: Company| async move {
                    company.id = Some(id.parse().unwrap());
                    s.repo
                        .replace_one(&company.clone().try_into().unwrap())
                        .await
                        .unwrap();
                    company.as_json_reply().boxed_infallible()
                },
            )
            .boxed()
    }
}
