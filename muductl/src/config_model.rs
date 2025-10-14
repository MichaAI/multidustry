use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub context: Context,
    pub servers: HashMap<String, Server>,
}

#[derive(Serialize, Deserialize)]
pub struct Context {
    pub current: String,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub apiserver_urls: Vec<String>,
}
