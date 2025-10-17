use std::sync::Arc;

use multidustry_kv::{KvStore, get_storage_instance};

pub async fn init_with_defaults() {
    let db = get_storage_instance().await;

    if let Ok(Some(data)) = db.get("defaults_inited").await {
        if &data == b"inited" {
            return;
        }
    }
    let _ = db.put("defaults_inited", b"inited".into()).await;

    let _ = db.put("config/server_name", b"Multidustry".into()).await;
    let _ = db.put("config/default_world_map_name", b"HUB".into()).await;
    let _ = db.put("config/version", b"-1".into());
    let _ = db.put("config/version_type", b"multidustry".into()).await;
    let _ = db.put("config/player_limit", b"0".into()).await;
    let _ = db
        .put(
            "config/description",
            b"Multidustry - best mindustry server implementation".into(),
        )
        .await;
    let _ = db.put("config/custom_gamemode", b"HUB".into()).await;

    let _ = db.put("stats/total_players", b"0".into()).await;
}

pub async fn get_string_from_db(db: &Arc<Box<dyn KvStore>>, key: &str) -> String {
    String::from_utf8(db.get(key).await.unwrap_or(None).unwrap_or(Vec::new())).unwrap_or_default()
}
