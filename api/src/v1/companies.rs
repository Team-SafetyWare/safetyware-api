use crate::common::{GetId, HasId, NewId, SetId};
use crate::crockford;
use crate::repo::company::Company as RepoCompany;
use crate::repo::company::CompanyRepo;
use crate::v1::op;
use crate::v1::ResourceApi;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::Reply;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
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
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl SetId for Company {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

impl NewId for Company {
    fn new_id() -> Self::Id {
        Some(crockford::random_id())
    }
}

#[derive(Clone)]
pub struct CompanyApi {
    pub repo: Arc<dyn CompanyRepo + Send + Sync + 'static>,
}

impl CompanyApi {
    pub fn new(repo: impl CompanyRepo + Send + Sync + 'static) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }
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
        op::replace::<Company, _, _>(self.collection_name(), self.repo.clone())
    }
}
