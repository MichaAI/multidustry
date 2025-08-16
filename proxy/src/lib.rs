use std::time::Duration;

use tokio;

mod udp_handler;

pub async fn proxy_main() {
    tokio::spawn(async {
        udp_handler::handle_udp_conections().await;
    });

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}

