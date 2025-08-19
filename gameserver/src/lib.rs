use std::time::Duration;

use multidustrycore::registry::{self, Component, ComponentType};
use tokio::sync::mpsc;

pub async fn gameserver_main() {
    let (tx, rx) = mpsc::channel(64);
    let _ = registry::register_service(
        Component {
            component_type: ComponentType::Gameserver,
            component_name: uuid::Uuid::new_v4().to_string(),
        },
        tx,
    );
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
