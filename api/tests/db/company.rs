use crate::db;
use api::db::company::{Company, CompanyRepo, MongoCompanyRepo};
use uuid::Uuid;

#[tokio::test]
async fn test_insert_one() {
    let db = db::db().await;
    let repo = MongoCompanyRepo::new(db);
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
    repo.delete_one(found.id).await.unwrap();
}
