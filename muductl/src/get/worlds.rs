use anyhow::Result;
use multidustrycore::api::v1::models::get::worlds::WorldsRes;
use tabled::{
    Table,
    settings::{Settings, Style},
};
use url::Url;

use crate::{config_model::Config, utils::find_healthy_apiserver};

pub async fn get_worlds(config: &Config) {
    let url = find_healthy_apiserver(&config).await;
    let _ = try_get_and_print_data(url).await;
}

async fn try_get_and_print_data(mut url: Url) -> Result<()> {
    url.set_path("v1/get/worlds");
    url.set_query(Some("fake=true"));

    let resp = reqwest::get(url).await?.json::<WorldsRes>().await?;

    let table_config = Settings::default().with(Style::rounded());

    let table = Table::new(resp.worlds).with(table_config).to_string();

    println!("{}", table);

    Ok(())
}
