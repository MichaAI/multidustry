use anyhow::Result;
use multidustrycore::api::v1::models::get::worlds::WorldsRes;
use tabled::{
    Table,
    settings::{Settings, Style},
};
use url::Url;

use crate::config_model::Config;

pub async fn get_worlds(config: &Config) {
    let server = &config.context.current;
    let apiserver_urls = config.servers.get(server).expect("Failed to get context");
    for apiserver_url in &apiserver_urls.apiserver_urls {
        let result = try_get_and_print_data(&apiserver_url).await;
        if let Ok(_) = result {
            return;
        }
    }
    println!("Failed to get data from any gameserver; Please check your network connection")
}

async fn try_get_and_print_data(url_str: &str) -> Result<()> {
    let mut url = Url::parse(url_str).expect("Failed to parce url");
    url.set_path("v1/get/worlds");
    url.set_query(Some("fake=true"));

    let resp = reqwest::get(url).await?.json::<WorldsRes>().await?;

    let table_config = Settings::default().with(Style::rounded());

    let table = Table::new(resp.worlds).with(table_config).to_string();

    println!("{}", table);

    Ok(())
}
