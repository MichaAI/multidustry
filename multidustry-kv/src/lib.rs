use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::OnceCell;
use tracing::{info, warn};
use url::Url;

use crate::{consul::ConsulKvStore, sled::SledKvStore};

pub mod consul;
pub mod sled;

#[async_trait]
pub trait KvStore: Send + Sync {
    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}

pub struct KvStorage;

pub static STORAGE_INSTANCE: OnceCell<Arc<Box<dyn KvStore>>> = OnceCell::const_new();
pub async fn get_storage_instance() -> &'static Arc<Box<dyn KvStore>> {
    STORAGE_INSTANCE
        .get_or_init(async || Arc::new(KvStorage::new().unwrap()))
        .await
}

impl KvStorage {
    fn new() -> Result<Box<dyn KvStore>> {
        if let Ok(path) = std::env::var("CONSUL_URL") {
            if let Ok(_) = Url::parse(&path) {
                info!("Using consul kv backend at {}", path);
                Ok(Box::new(ConsulKvStore::new(Client::new(), path)?))
            } else {
                warn!(
                    "Failed to parce consul url. Using sled kv backend at {:?}",
                    get_kv_path()
                );
                Ok(Box::new(SledKvStore::new(&get_kv_path())?))
            }
        } else {
            warn!("Using sled kv backend at {:?}", get_kv_path());
            Ok(Box::new(SledKvStore::new(&get_kv_path())?))
        }
    }
}

fn get_kv_path() -> PathBuf {
    let mut data_path = std::env::var("MULTIDUSTRY_SLED_PATH")
        .map(|path| PathBuf::from(path))
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    data_path.push("multidustry.sled");
    data_path
}
