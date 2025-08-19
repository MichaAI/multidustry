use multidustrycore::{
    registry::{Component, ComponentType},
    *,
};
use std::time::Duration;

pub async fn apiserver_main() {
    let (tx, rx) = tokio::sync::mpsc::channel(64);
    let _ = registry::register_service(
        Component {
            component_type: ComponentType::Apiserver,
            component_name: uuid::Uuid::new_v4().to_string(),
        },
        tx,
    );
    println!("Apiserver started");
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
