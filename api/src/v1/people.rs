use crate::common::{GetId, HasId, NewId, SetId};
use crate::crockford;
use crate::repo::person::Person as RepoPerson;
use crate::repo::person::PersonRepo;
use crate::v1::op;
use crate::v1::ResourceApi;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::Reply;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<String>,
    pub name: String,
    pub company_id: String,
}

impl From<RepoPerson> for Person {
    fn from(value: RepoPerson) -> Self {
        Self {
            id: Some(value.id.to_string()),
            name: value.name,
            company_id: value.company_id,
        }
    }
}

impl TryFrom<Person> for RepoPerson {
    type Error = anyhow::Error;

    fn try_from(value: Person) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.context("id missing")?.parse()?,
            name: value.name,
            company_id: value.company_id,
        })
    }
}

impl HasId for Person {
    type Id = Option<String>;
}

impl GetId for Person {
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl SetId for Person {
    fn set_id(&mut self, id: Self::Id) {
        self.id = id
    }
}

impl NewId for Person {
    fn new_id() -> Self::Id {
        Some(crockford::random_id())
    }
}

#[derive(Clone)]
pub struct PersonApi {
    pub repo: Arc<dyn PersonRepo + Send + Sync + 'static>,
}

impl PersonApi {
    pub fn new(repo: impl PersonRepo + Send + Sync + 'static) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }
}

impl ResourceApi for PersonApi {
    fn collection_name(&self) -> String {
        "people".to_string()
    }

    fn get(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::get::<Person, _, _>(self.collection_name(), self.repo.clone())
    }

    fn list(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::list::<Person, _, _>(self.collection_name(), self.repo.clone())
    }

    fn create(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::create::<Person, _, _>(self.collection_name(), self.repo.clone())
    }

    fn delete(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::delete(self.collection_name(), self.repo.clone())
    }

    fn replace(&self) -> BoxedFilter<(Box<dyn Reply>,)> {
        op::replace::<Person, _, _>(self.collection_name(), self.repo.clone())
    }
}
