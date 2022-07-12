use crate::{
    client::{Client, ClientType, Entity},
    world::{WorldServerInfo, WorldStatus},
};
use aw_core::{AWPacket, VarID};

pub fn world_server_start(client: &Client, packet: &AWPacket) {
    if let Some(client_type) = client.info().client_type {
        log::warn!(
            "A client who already has type {:?} tried to start a world server.",
            client_type
        );
        return;
    }

    let _browser_version = packet.get_int(VarID::BrowserVersion);
    let world_build = packet.get_int(VarID::WorldBuild);
    let world_port = packet.get_int(VarID::WorldPort);

    if let (Some(world_build), Some(world_port)) = (world_build, world_port) {
        let client_entity = Entity::WorldServer(WorldServerInfo {
            build: world_build,
            server_port: world_port as u16,
            worlds: Vec::new(),
        });

        client.info_mut().client_type = Some(ClientType::World);
        client.info_mut().entity = Some(client_entity);

        log::info!("World server {} connected.", client.addr.ip());
    }
}

pub fn world_server_hide_all(server: &mut WorldServerInfo) {
    for world in &mut server.worlds {
        world.status = WorldStatus::Hidden;
    }
}
