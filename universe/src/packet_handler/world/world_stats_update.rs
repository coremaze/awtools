use aw_core::{AWPacket, VarID};

use crate::{
    get_conn_mut, tabs::regenerate_world_list, universe_connection::UniverseConnectionID,
    world::WorldRating, UniverseServer,
};

#[derive(Debug)]
enum WorldStatsUpdateParamsError {
    Rating,
    FreeEntry,
    UserCount,
    Name,
}

struct WorldStatsUpdateParams {
    world_rating: WorldRating,
    world_free_entry: bool,
    world_user_count: u32,
    world_name: String,
}

impl TryFrom<&AWPacket> for WorldStatsUpdateParams {
    type Error = WorldStatsUpdateParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let world_rating = value
            .get_byte(VarID::WorldRating)
            .and_then(WorldRating::from_u8)
            .ok_or(WorldStatsUpdateParamsError::Rating)?;
        let world_free_entry = value
            .get_byte(VarID::WorldFreeEntry)
            .map(|free_entry| free_entry != 0)
            .ok_or(WorldStatsUpdateParamsError::FreeEntry)?;
        let world_user_count = value
            .get_uint(VarID::WorldUsers)
            .ok_or(WorldStatsUpdateParamsError::UserCount)?;
        let world_name = value
            .get_string(VarID::WorldName)
            .ok_or(WorldStatsUpdateParamsError::Name)?;

        Ok(Self {
            world_rating,
            world_free_entry,
            world_user_count,
            world_name,
        })
    }
}

pub fn world_stats_update(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let params = match WorldStatsUpdateParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete world stats update: {why:?}");
            return;
        }
    };

    let conn = get_conn_mut!(server, cid, "world_stats_update");

    let Some(world_server) = conn.world_server_mut() else {
        return;
    };

    let Some(world) = world_server.get_world_mut(&params.world_name) else {
        return;
    };

    world.rating = params.world_rating;
    world.free_entry = params.world_free_entry;
    world.user_count = params.world_user_count;

    // Change world information in everyone's world list
    for cid in server.connections.cids() {
        regenerate_world_list(server, cid)
    }
}
