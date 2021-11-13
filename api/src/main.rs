use std::net::Ipv4Addr;
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    warp::serve(hello)
        .run((Ipv4Addr::UNSPECIFIED, 3001))
        .await;
}
