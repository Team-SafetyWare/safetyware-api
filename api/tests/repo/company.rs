use crate::repo;
use api::repo::company::{Company, CompanyRepo, MongoCompanyRepo};
use std::future::Future;
use uuid::Uuid;

#[tokio::test]
async fn test_insert_one() {
    test_repo(|repo| async move {
        let company = Company {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };

        repo.insert_one(&company).await.unwrap();

        let opt = repo.find_one(company.id).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found.id, company.id);
        assert_eq!(found.name, company.name);
    })
    .await
}

async fn test_repo<T, F>(test: T)
where
    T: Fn(MongoCompanyRepo) -> F,
    F: Future,
{
    let db = repo::new_db().await.unwrap();
    let repo = MongoCompanyRepo::new(db.clone());
    test(repo).await;
    db.drop(None).await.unwrap();
}
