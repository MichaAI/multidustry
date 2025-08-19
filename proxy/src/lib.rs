use std::{cell::Ref, time::Duration};

use multidustrycore::registry::{self, Component, ComponentType};
use tokio::{self, sync::mpsc};

mod udp_handler;

pub async fn proxy_main() {
    let (tx, rx) = mpsc::channel(64);
    let _ = registry::register_service(
        Component {
            component_type: ComponentType::Proxy,
            component_name: uuid::Uuid::new_v4().to_string(),
        },
        tx,
    );

    tokio::spawn(async {
        udp_handler::handle_udp_conections().await;
    });

    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
