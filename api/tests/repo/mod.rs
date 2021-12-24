use mongodb::{Client, Database};
use tokio::sync::Mutex;
use uuid::Uuid;

mod company;

lazy_static::lazy_static! {
    static ref DB_CLIENT: Mutex<Option<Client>> = Mutex::new(None);
}

pub async fn new_db() -> anyhow::Result<Database> {
    let client = db_client().await?;
    let name = format!("test-{}", Uuid::new_v4());
    let db = client.database(&name);
    Ok(db)
}

pub async fn db_client() -> anyhow::Result<Client> {
    let mut client_shared = DB_CLIENT.lock().await;
    let client = if let Some(client) = &*client_shared {
        client.clone()
    } else {
        let client = Client::with_uri_str("mongodb://localhost:42781").await?;
        *client_shared = Some(client.clone());
        client
    };
    Ok(client)
}
