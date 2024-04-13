use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    get_conn_mut, tabs::regenerate_world_list, universe_connection::UniverseConnectionID,
    UniverseServer,
};

#[derive(Debug)]
enum WorldStopParamsError {
    NoName,
}

struct WorldStopParams {
    world_name: String,
}

impl TryFrom<&AWPacket> for WorldStopParams {
    type Error = WorldStopParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let world_name = value
            .get_string(VarID::WorldName)
            .ok_or(WorldStopParamsError::NoName)?;

        Ok(WorldStopParams { world_name })
    }
}

pub fn world_stop(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let params = match WorldStopParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete world stop: {why:?}");
            return;
        }
    };

    let world_exists = server
        .connections
        .get_world_by_name(&params.world_name)
        .is_some();

    let conn = get_conn_mut!(server, cid, "world_stop");

    let Some(world_server) = conn.world_server_mut() else {
        return;
    };

    // Remove the world from the client
    log::trace!("Before remove: {world_server:?}");
    let removed_world = world_server.remove_world(&params.world_name);
    log::trace!("After remove: {world_server:?}");
    log::trace!("{removed_world:?}");

    let rc = match world_exists {
        true => match removed_world {
            Some(_) => ReasonCode::Success,
            None => ReasonCode::NotWorldOwner,
        },
        false => ReasonCode::NoSuchWorld,
    };

    let mut p = AWPacket::new(PacketType::WorldStop);

    p.add_int(VarID::ReasonCode, rc as i32);

    conn.send(p);

    // Remove world from clients' world list
    for cid in server.connections.cids() {
        regenerate_world_list(server, cid)
    }
}
