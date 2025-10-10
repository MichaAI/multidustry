use crate::KvStore;
use anyhow::Result;
use async_trait::async_trait;

pub struct ConsulKvStore {
    client: reqwest::Client,
    base_url: String,
}

impl ConsulKvStore {
    pub fn new(client: reqwest::Client, base_url: String) -> Result<Self> {
        Ok(Self { client, base_url })
    }
}

#[async_trait]
impl KvStore for ConsulKvStore {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
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

        let bytes = response.error_for_status()?.bytes().await?;
        Ok(Some(bytes.to_vec()))
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
