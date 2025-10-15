use std::{sync::Arc, time::Duration};

use crate::KvStore;
use anyhow::Result;
use async_trait::async_trait;
use dashmap::{DashMap, Map};
use tracing::debug;

pub struct ConsulKvStore {
    client: reqwest::Client,
    base_url: String,
    cache: Arc<DashMap<String, CacheEntry>>,
}

pub struct CacheEntry {
    pub data: Vec<u8>,
    pub index: u64,
}

impl ConsulKvStore {
    pub fn new(client: reqwest::Client, base_url: String) -> Result<Self> {
        Ok(Self {
            client,
            base_url,
            cache: Arc::new(DashMap::new()),
        })
    }

    pub async fn watch_key(&self, key: &str) {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let cache = Arc::clone(&self.cache);
        let key = key.to_string().clone();
        tokio::spawn(async move {
            let mut last_index: u64 = 0;
            loop {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        debug!("Finishing watching for key: {}", key)
                    }
                    Ok(res) = Self::blocking_get(&client, &base_url, &key, last_index) => {
                        match res {
                            None => { // Key removed
                                cache.remove(&key);
                            },
                            Some((data, new_index)) => {
                                if new_index > last_index {
                                    debug!("Cache updated for key: {} (index: {})", key, new_index);
                                    cache.insert(key.clone(), CacheEntry {
                                        data,
                                        index: new_index,
                                    });
                                    last_index = new_index;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn blocking_get(
        client: &reqwest::Client,
        base_url: &str,
        key: &str,
        wait_index: u64,
    ) -> Result<Option<(Vec<u8>, u64)>> {
        let url = format!("{}/{}", base_url, key);

        let mut request = client
            .get(&url)
            .query(&[("raw", "true")])
            .timeout(Duration::from_secs(65)); // Consul default timeout + buffer

        if wait_index > 0 {
            request = request.query(&[("index", wait_index.to_string())]);
        }

        let response = request.send().await?;

        if response.status() == 404 {
            return Ok(None);
        }

        // Extract X-Consul-Index header
        let consul_index = response
            .headers()
            .get("X-Consul-Index")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let bytes = response.error_for_status()?.bytes().await?;
        Ok(Some((bytes.to_vec(), consul_index)))
    }
}

#[async_trait]
impl KvStore for ConsulKvStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(cache_entry) = self.cache.get(key) {
            let res = cache_entry.value();
            return Ok(Some(res.data));
        }

        let url = format!("{}/{}", self.base_url, key);

        let response = self
            .client
            .get(&url)
            .query(&[("raw", "true")]) // Получаем raw value без метаданных
            .send()
            .await?;

        if response.status() == 404 {
            return Ok(None);
        }

        let bytes = response.error_for_status()?.bytes().await?.to_vec();
        self.cache.insert(
            key.into(),
            CacheEntry {
                data: bytes.clone(),
                index: 0,
            },
        );

        Ok(Some(bytes))
    }

    async fn put(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let url = format!("{}/{}", self.base_url, key);

        self.client
            .put(&url)
            .body(value)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let url = format!("{}/{}", self.base_url, key);

        self.client.delete(&url).send().await?.error_for_status()?;
        self.cache.remove(key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let url = format!("{}/{}", self.base_url, prefix);

        let response = self
            .client
            .get(&url)
            .query(&[("keys", "true")]) // Получаем только ключи
            .send()
            .await?;

        if response.status() == 404 {
            return Ok(vec![]);
        }

        let keys: Vec<String> = response.error_for_status()?.json().await?;

        Ok(keys)
    }
}
