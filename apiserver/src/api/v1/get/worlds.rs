use fake::{Fake, rand};
use multidustrycore::api::v1::models::get::worlds::{World, WorldsRes};
use rocket::{get, serde::json::Json};
use tracing::info;

#[get("/v1/get/worlds?<fake>")]
pub async fn get_worlds(fake: bool) -> Json<WorldsRes> {
    info!("{}", fake);
    if fake {
        let worlds = (1..10).map(|_| World::fake()).collect();

        Json(WorldsRes::new(worlds))
    } else {
        Json(WorldsRes::new(Vec::new()))
    }
}
