use binrw::binrw;
use bon::Builder;

use crate::protocol::networkio::string::NetworkIOString;

#[binrw]
#[brw(big)]
#[derive(Builder)]
pub struct DiscoveryResponce {
    server_name: NetworkIOString,
    map_name: NetworkIOString,
    total_players: i32,
    wave: i32,
    #[builder(default = -1)]
    version: i32,
    version_type: NetworkIOString,
    gamemode: i8,
    #[builder(default = 0)]
    player_limit: i32,
    description: NetworkIOString,
    custom_gamemode: NetworkIOString,
    #[builder(default = 6567)]
    server_port: i16,
}
