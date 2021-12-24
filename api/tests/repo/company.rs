use crate::repo;
use api::repo::company::{Company, CompanyRepo, MongoCompanyRepo};
use uuid::Uuid;

#[tokio::test]
async fn test_insert_one() {
    let repo = repo().await.unwrap();
    let company = Company {
        id: Default::default(),
        name: Uuid::new_v4().to_string(),
    };

    repo.insert_one(&company).await.unwrap();

    let found = repo.find_one(company.id).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, company.id);
    assert_eq!(found.name, company.name);
}

async fn repo() -> anyhow::Result<impl CompanyRepo> {
    let db = repo::new_db().await?;
    let repo = MongoCompanyRepo::new(db);
    Ok(repo)
}
