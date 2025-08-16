use gameserver::gameserver_main;
use tokio;

#[tokio::main]
async fn main() {
    gameserver_main().await;
}
