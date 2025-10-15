use std::time::Duration;

use tokio::sync::mpsc;
use tracing::info;

pub async fn gameserver_main() {
    info!("Gameserver started, waiting ctrl+c...");
    let _ = tokio::signal::ctrl_c().await;
    info!("Exiting gameserver...");
    // Todo: migrate all worlds
}
