use std::fs::File;
use std::io::{self, Read};

use serde::Deserialize;
use toml;

const CONFIG_PATH: &'static str = "src/.backup-cred-server.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    secret: String,
    port: u16,
    pub api_keys: Option<Vec<String>>,
}

impl Config {
    pub fn get() -> Result<Self, Error> {
        let mut file = String::new();
        File::open(CONFIG_PATH)?.read_to_string(&mut file)?;

        let config: Self = toml::from_str(&file)?;
        Ok(config)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn key(&self) -> String {
        self.secret.clone()
    }
}

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    Io(io::Error),
    #[allow(dead_code)]
    Parsing(toml::de::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::Parsing(value)
    }
}
