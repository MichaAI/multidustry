use multidustrycore::api::v1::models::kv::set::SetRes;
use reqwest::Client;
use tabled::{Table, settings::Style};

use crate::{config_model::Config, utils::find_healthy_apiserver};

pub async fn set(config: &Config, key: &str, value: &str) -> anyhow::Result<()> {
    let mut url = find_healthy_apiserver(config).await;
    url.set_path("v1/kv/set");
    let client = Client::new();
    let resp = client
        .get(url)
        .query(&[("key", key), ("value", value)])
        .build()?;
    let result = client.execute(resp).await?.json::<SetRes>().await?;

    let tabled_res = [result];

    println!("{}", Table::new(tabled_res).with(Style::rounded()));

    Ok(())
}
