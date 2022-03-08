use crate::repo::user_account::ArcUserAccountRepo;
use crate::warp_ext;
use mongodb::Database;
use warp::filters::BoxedFilter;
use warp::http::header::CONTENT_TYPE;
use warp::http::HeaderValue;
use warp::reply::Response;
use warp::{Filter, Reply};

#[derive(Clone)]
pub struct Context {
    pub user_account_repo: ArcUserAccountRepo,
    pub db: Database,
}

pub fn v1(context: Context) -> BoxedFilter<(impl Reply,)> {
    warp::path("v1")
        .and(health(context.db).or(user_account_profile_image(context.user_account_repo)))
        .boxed()
}

fn health(db: Database) -> BoxedFilter<(impl Reply,)> {
    warp::path("health")
        .and(warp_ext::with_clone(db))
        .then(move |db: Database| async move {
            crate::db::test_connection(&db).await?;
            Ok(warp::reply())
        })
        .map(warp_ext::convert_err)
        .boxed()
}

fn user_account_profile_image(user_account_repo: ArcUserAccountRepo) -> BoxedFilter<(impl Reply,)> {
    warp::path!("userAccount" / String / "profile.png")
        .and(warp_ext::with_clone(user_account_repo))
        .then(
            move |user_account_id: String, user_account_repo: ArcUserAccountRepo| async move {
                // Todo: Use default image if none.
                let png_bytes = user_account_repo
                    .profile_image_png(&user_account_id)
                    .await?
                    .expect("profile image missing");
                let mut response = Response::new(png_bytes.into());
                response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("image/png"));
                Ok(response)
            },
        )
        .map(warp_ext::convert_err)
        .boxed()
}
