use crate::common::{GetId, HasId, NewId, SetId};
use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::v1::op;
use crate::v1::{ResourceApi, ResourceOperation};
use crate::warp_ext::{AsJsonReply, BoxReplyInfallible};
use anyhow::Context;
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;
use warp::filters::BoxedFilter;
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

impl NewId for Company {
    fn new_id() -> Self::Id {
        Some(ObjectId::new().to_string())
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
        op::get::<Company, _, _>(self.collection_name(), self.repo.clone())
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::list::<Company, _, _>(self.collection_name(), self.repo.clone())
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::create::<Company, _, _>(self.collection_name(), self.repo.clone())
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::delete(self.collection_name(), self.repo.clone())
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
