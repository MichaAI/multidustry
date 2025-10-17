use multidustrycore::{kv, observ::init_observ};
use proxy::proxy_main;
use tokio;

#[tokio::main]
async fn main() {
    let _ = init_observ();
    kv::init_with_defaults().await;
    proxy_main().await;
}
