use crate::graphql::{encode_query, test_graphql};
use api::crockford;
use api::repo::company::Company;
use api::repo::op::InsertOne;
use json_dotpath::DotPaths;
use serde_json::Value;
use warp::http::header::CONTENT_TYPE;

#[tokio::test]
async fn test_get_company() {
    test_graphql(|filter, context| async move {
        // Arrange.
        let company = Company {
            id: crockford::random_id(),
            name: crockford::random_id(),
        };
        context.company_repo.insert_one(&company).await?;

        // Act.
        let res = warp::test::request()
            .method("POST")
            .path("/graphql")
            .header(CONTENT_TYPE, "application/json")
            .body(encode_query(format!(
                r#"
                    query {{
                        company(id: "{}") {{
                            id
                            name
                        }}
                    }}
                "#,
                company.id
            ))?)
            .reply(&filter)
            .await;

        // Assert.
        let json: Value = serde_json::from_slice(res.body())?;
        assert_eq!(json.dot_get::<String>("data.company.id")?, Some(company.id));
        assert_eq!(
            json.dot_get::<String>("data.company.name")?,
            Some(company.name)
        );
        Ok(())
    })
    .await
}
