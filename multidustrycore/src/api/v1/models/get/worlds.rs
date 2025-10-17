use bon::Builder;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tabled::Tabled;

#[derive(Serialize, Deserialize)]
pub struct WorldsRes {
    pub worlds: Vec<World>,
}

impl WorldsRes {
    pub fn new(worlds: Vec<World>) -> Self {
        Self { worlds }
    }
}

#[derive(Tabled, Builder, Serialize, Deserialize)]
pub struct World {
    id: String,
    name: String,
    status: WorldStatus,
    player_count: String,
    gameserver: String,
    mode: String,
    uptime: String,
    map_name: String,

    #[tabled(skip)]
    plugins: Vec<String>,
    #[tabled(skip)]
    detail: WorldDetail,
}

#[derive(Builder, Serialize, Deserialize)]
pub struct WorldDetail {
    map_size: String,
    // plugins: Vec<PluginInfo>,
    gameserver_ip: String,
    resource_usage: ResourceUsage,
    created_at: String,
    last_save: String,
    // player_list: Vec<PlayerInfo>,  // Список подключенных игроков
}

#[derive(Builder, Serialize, Deserialize)]
pub struct ResourceUsage {
    cpu_percent: f32,
    memory_mb: u64,
    tick_rate: f32, // TPS (ticks per second)
}

#[derive(Tabled, Serialize, Deserialize, Clone, Copy)]
pub enum WorldStatus {
    Running,
    Migrating,
    Stopped,
    Failed,
}

impl Display for WorldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Running => write!(f, "{}", " Run".green()),
            Self::Migrating => write!(f, "{}", "󰑓 Mig".cyan()),
            Self::Stopped => write!(f, "{}", " Stop".dimmed()),
            Self::Failed => write!(f, "{}", " Fail".red()),
        }
    }
}

use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::IPv4;
use fake::faker::lorem::en::Word;
use fake::faker::name::en::Name;
use fake::faker::number::en::NumberWithFormat;
use fake::{Fake, Faker};
use rand::Rng;
use rand::seq::{IndexedRandom, SliceRandom};

impl World {
    pub fn fake() -> Self {
        let mut rng = rand::thread_rng();

        // Random world status
        let status = *[
            WorldStatus::Running,
            WorldStatus::Migrating,
            WorldStatus::Stopped,
            WorldStatus::Failed,
        ]
        .choose(&mut rng)
        .unwrap();

        // Generate world ID (like w_abc123def)
        let world_id: String = format!(
            "w_{}",
            NumberWithFormat("^#^#^#^#^#")
                .fake::<String>()
                .to_lowercase()
        );

        // Generate world name (something like "Survival Valley" or "PvP Arena")
        let name_parts = vec![
            vec![
                "Survival",
                "Creative",
                "PvP",
                "Hardcore",
                "Peaceful",
                "Adventure",
            ],
            vec![
                "Valley", "Arena", "Island", "Fortress", "World", "Realm", "Domain",
            ],
        ];
        let world_name = format!(
            "{} {}",
            name_parts[0].choose(&mut rng).unwrap(),
            name_parts[1].choose(&mut rng).unwrap()
        );

        // Player count (like "12/50")
        let current_players: u16 = rng.gen_range(0..100);
        let max_players: u16 = rng.gen_range(current_players.max(10)..150);
        let player_count = format!("{}/{}", current_players, max_players);

        // Gameserver name
        let gameserver: String = format!(
            "gs-{}",
            CompanyName()
                .fake::<String>()
                .to_lowercase()
                .replace(" ", "-")
        );

        // Game mode
        let mode = *["survival", "creative", "adventure", "spectator", "hardcore"]
            .choose(&mut rng)
            .unwrap();

        // Uptime (like "3d 14h 27m")
        let days = rng.gen_range(0..30);
        let hours = rng.gen_range(0..24);
        let minutes = rng.gen_range(0..60);
        let uptime = if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        };

        // Map name (procedurally generated)
        let map_name: String = format!("map_{}", (1000000..9999999).fake::<u32>());

        // Generate random plugins
        let plugin_names = vec![
            "WorldEdit",
            "EssentialsX",
            "Vault",
            "LuckPerms",
            "CoreProtect",
            "WorldGuard",
            "Citizens",
            "Multiverse-Core",
            "PlaceholderAPI",
            "ProtocolLib",
        ];
        let plugin_count = rng.gen_range(3..8);
        let plugins: Vec<String> = plugin_names
            .choose_multiple(&mut rng, plugin_count)
            .map(|s| s.to_string())
            .collect();

        // Generate WorldDetail
        let detail = WorldDetail::fake();

        Self {
            id: world_id,
            name: world_name,
            status,
            player_count,
            gameserver,
            mode: mode.to_string(),
            uptime,
            map_name,
            plugins,
            detail,
        }
    }
}

impl WorldDetail {
    pub fn fake() -> Self {
        let mut rng = rand::thread_rng();

        // Map size in blocks (like "10000x10000")
        let size = rng.gen_range(5000..30000);
        let map_size = format!("{}x{}", size, size);

        // Gameserver IP
        let gameserver_ip: String = format!(
            "{}:{}",
            IPv4().fake::<String>(),
            rng.gen_range(25000..26000)
        );

        // Resource usage
        let resource_usage = ResourceUsage::fake();

        // Timestamps (ISO 8601 format)
        use chrono::{Duration, Utc};
        let created_days_ago = rng.gen_range(1..365);
        let created_at = (Utc::now() - Duration::days(created_days_ago))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        let last_save_minutes_ago = rng.gen_range(1..60);
        let last_save = (Utc::now() - Duration::minutes(last_save_minutes_ago))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        Self {
            map_size,
            gameserver_ip,
            resource_usage,
            created_at,
            last_save,
        }
    }
}

impl ResourceUsage {
    pub fn fake() -> Self {
        let mut rng = rand::thread_rng();

        Self {
            cpu_percent: rng.gen_range(5.0..95.0),
            memory_mb: rng.gen_range(512..8192),
            tick_rate: rng.gen_range(15.0..60.0), // Normal TPS is ~20
        }
    }
}
