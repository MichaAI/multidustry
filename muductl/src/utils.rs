use core::panic;

use reqwest::header::CONTENT_ENCODING;
use url::Url;

use crate::config_model::Config;

pub async fn find_healthy_apiserver(config: &Config) -> Url {
    let server = &config.context.current;
    let apiserver_urls = config.servers.get(server).expect("Failed to get context");
    for apiserver_url in &apiserver_urls.apiserver_urls {
        match Url::parse(&apiserver_url) {
            Ok(url) => {
                let mut check_url = url.clone();
                check_url.set_path("/ping");
                match reqwest::get(check_url).await {
                    Ok(data) => match data.text().await {
                        Ok(resp) => {
                            if &resp != "pong" {
                                eprintln!("Server answered not pong");
                                continue;
                            } else {
                                return url;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to get status of apiserver at {}: {}",
                                &apiserver_url, e
                            );
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            "Failed to get status of apiserver at {}: {}",
                            &apiserver_url, e
                        );
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to parce url {}: {}", &apiserver_url, e);
                continue;
            }
        }
    }
    panic!("Failed to find any healthy apiserver! Exiting.")
}
