use crate::repo;
use api::repo::company::{Company, CompanyRepo, MongoCompanyRepo};
use futures_util::TryStreamExt;
use std::future::Future;
use uuid::Uuid;

#[tokio::test]
async fn test_insert_one() {
    test_repo(|repo| async move {
        // Arrange.
        let company = Company {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };

        // Act.
        repo.insert_one(&company).await.unwrap();

        // Assert.
        let opt = repo.find_one(company.id).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found.id, company.id);
        assert_eq!(found.name, company.name);
    })
    .await
}

#[tokio::test]
async fn test_replace_one() {
    test_repo(|repo| async move {
        // Arrange.
        let id = Default::default();
        let first = Company {
            id,
            name: Uuid::new_v4().to_string(),
        };
        repo.insert_one(&first).await.unwrap();
        let second = Company {
            id,
            name: Uuid::new_v4().to_string(),
        };

        // Act.
        repo.replace_one(&second).await.unwrap();

        // Assert.
        let opt = repo.find_one(id).await.unwrap();
        let found = opt.expect("not found");
        assert_eq!(found.name, second.name);
    })
    .await
}

#[tokio::test]
async fn test_find_one() {
    test_repo(|repo| async move {
        // Arrange.
        let company = Company {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };
        repo.insert_one(&company).await.unwrap();

        // Act.
        let opt = repo.find_one(company.id).await.unwrap();

        // Assert.
        let found = opt.expect("not found");
        assert_eq!(found.id, company.id);
        assert_eq!(found.name, company.name);
    })
    .await
}

#[tokio::test]
async fn test_find() {
    test_repo(|repo| async move {
        // Arrange.
        let companies: Vec<_> = (0..3)
            .map(|_| Company {
                id: Default::default(),
                name: Uuid::new_v4().to_string(),
            })
            .collect();
        for company in &companies {
            repo.insert_one(company).await.unwrap();
        }

        // Act.
        let stream = repo.find().await.unwrap();

        // Assert.
        let found: Vec<_> = stream.try_collect().await.unwrap();
        assert_eq!(found.len(), companies.len());
        for company in companies {
            assert!(found.iter().any(|c| c.id == company.id))
        }
    })
    .await
}

#[tokio::test]
async fn test_delete_one() {
    test_repo(|repo| async move {
        let company = Company {
            id: Default::default(),
            name: Uuid::new_v4().to_string(),
        };
        repo.insert_one(&company).await.unwrap();

        repo.delete_one(company.id).await.unwrap();

        let opt = repo.find_one(company.id).await.unwrap();
        assert!(opt.is_none());
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
