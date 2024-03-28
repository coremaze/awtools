use crate::{
    client::ClientInfo, get_conn_mut, universe_connection::UniverseConnectionID,
    world::WorldServer, UniverseServer,
};
use aw_core::{AWPacket, VarID};

pub fn world_server_start(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let conn = get_conn_mut!(server, cid, "world_server_start");
    if conn.client.is_some() {
        log::warn!(
            "A connection who already has client {:?} tried to start a world server.",
            conn.client
        );
        return;
    }

    let _browser_version = packet.get_int(VarID::BrowserVersion);
    let world_build = packet.get_int(VarID::WorldBuild);
    let world_port = packet.get_int(VarID::WorldPort);

    if let (Some(world_build), Some(world_port)) = (world_build, world_port) {
        conn.client = Some(ClientInfo::WorldServer(WorldServer {
            build: world_build,
            server_port: world_port as u16,
            worlds: Vec::new(),
        }));

        log::info!(
            "Connection {} has made itself a world server.",
            conn.addr().ip()
        );
    }
}
