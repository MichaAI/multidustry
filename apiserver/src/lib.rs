use std::time::Duration;

pub mod api;

use rocket::{get, routes};

pub async fn apiserver_main() {
    tokio::spawn(async {
        rocket::build()
            .mount("/", routes![ping, api::v1::get::worlds::get_worlds])
            .launch()
            .await
    });

    println!("Apiserver started");
    let _ = tokio::signal::ctrl_c().await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Apiserver exited");
}

#[get("/ping")]
async fn ping() -> &'static str {
    "pong"
}
