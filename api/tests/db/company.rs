use crate::db;
use api::db::company::Company;
use uuid::Uuid;

#[tokio::test]
async fn test_upsert_insert() {
    let db = db::db().await;
    let name = Uuid::new_v4().to_string();
    let company = Company {
        id: Default::default(),
        name,
    };

    company.upsert(&db).await.unwrap();

    let found = Company::find_one(company.id, &db).await.unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.name, company.name);
    found.delete(&db).await.unwrap();
}
