use apiserver::apiserver_main;
use multidustrycore::observ::init_observ;
use tokio;

#[tokio::main]
async fn main() {
    let _ = init_observ();
    apiserver_main().await;
}
