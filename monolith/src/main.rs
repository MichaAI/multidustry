use apiserver::apiserver_main;
use gameserver::gameserver_main;
use tokio;
use proxy::proxy_main;

#[tokio::main]
async fn main() {
    let apiserver = tokio::spawn(async {
        apiserver_main().await;
    });

    let proxy = tokio::spawn(async {
        proxy_main().await;
    });

    let gameserver = tokio::spawn(async {
        gameserver_main().await;
    });

    tokio::select! {
        _ = apiserver => println!("Apiserver exited"),
        _ = proxy => println!("Proxy exited"),
        _ = gameserver => println!("Gameserver exited"),
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
        }
    }
}
