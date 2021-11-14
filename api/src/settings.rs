use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub db_uri: String,
}

impl Settings {
    pub fn read() -> Self {
        Config::default()
            .with_merged(Environment::with_prefix("sw"))
            .unwrap()
            .try_into()
            .unwrap()
    }
}
