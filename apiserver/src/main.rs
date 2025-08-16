use apiserver::apiserver_main;
use tokio;

#[tokio::main]
async fn main() {
    apiserver_main().await;
}
