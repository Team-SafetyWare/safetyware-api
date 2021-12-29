use crate::repo;
use api::crockford;
use api::db::coll;
use api::repo::location_reading::{
    Location, Metadata, MongoLocationReading, MongoLocationReadingRepo,
};
use api::repo::op::Find;
use chrono::{DateTime, NaiveDate, Utc};
use futures_util::TryStreamExt;
use mongodb::Database;
use std::future::Future;

#[tokio::test]
async fn test_list() {
    test_op(|db| async move {
        // Arrange.
        let reading = MongoLocationReading {
            timestamp: DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0),
                Utc,
            ),
            metadata: Metadata {
                person_id: crockford::random_id(),
            },
            location: Location {
                coordinates: vec![-60.0, 40.0],
            },
        };
        let readings = vec![reading.clone(), reading.clone()];
        db.collection(coll::LOCATION_READING)
            .insert_many(readings, None)
            .await
            .unwrap();
        let repo = MongoLocationReadingRepo::new(db);

        // Act.
        let stream = repo.find().await.unwrap();

        // Assert.
        let found: Vec<_> = stream.try_collect().await.unwrap();
        assert_eq!(found.len(), 2);
        for f in found {
            assert_eq!(f.timestamp, reading.timestamp.to_string());
            assert_eq!(f.person_id, reading.metadata.person_id);
            assert_eq!(f.coordinates, reading.location.coordinates);
        }
    })
    .await
}

async fn test_op<T, F>(test: T)
where
    T: Fn(Database) -> F,
    F: Future,
{
    let db = repo::new_db().await.unwrap();
    test(db.clone()).await;
    db.drop(None).await.unwrap();
}
