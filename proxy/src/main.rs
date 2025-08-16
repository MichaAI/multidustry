use tokio;
use proxy::proxy_main;

#[tokio::main]
async fn main() {
    proxy_main().await;
}
