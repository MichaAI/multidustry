use std::{cell::Ref, time::Duration};

use tokio::{self, sync::mpsc};

mod protocol;
mod tcp_handler;
mod udp_handler;

pub async fn proxy_main() {
    tokio::spawn(async {
        udp_handler::handle_udp_conections().await;
    });

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
