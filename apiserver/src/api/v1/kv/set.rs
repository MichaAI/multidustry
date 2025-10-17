use multidustry_kv::get_storage_instance;
use multidustrycore::api::v1::models::kv::set::SetRes;
use rocket::{get, serde::json::Json};

#[get("/v1/kv/set?<key>&<value>")]
pub async fn set(key: &str, value: &str) -> Json<SetRes> {
    let db = get_storage_instance().await;
    let _ = db.put(key, value.into()).await;
    Json(SetRes::builder().ok(true).build())
}
