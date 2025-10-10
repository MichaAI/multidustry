use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;

use crate::KvStore;

pub struct SledKvStore {
    db: sled::Db,
}

impl SledKvStore {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db: db })
    }
}

#[async_trait]
impl KvStore for SledKvStore {
    async fn put(&self, key: &str, data: Vec<u8>) -> Result<()> {
        self.db.insert(key, data);
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key.as_bytes())?.map(|v| v.to_vec()))
    }
    async fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key);
        Ok(())
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        Ok(self
            .db
            .scan_prefix(prefix.as_bytes())
            .keys()
            .filter_map(|key| key.ok())
            .filter_map(|key| String::from_utf8(key.to_vec()).ok())
            .collect())
    }
}
