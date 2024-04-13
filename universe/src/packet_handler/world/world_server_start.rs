use crate::{
    client::ClientInfo, get_conn_mut, universe_connection::UniverseConnectionID,
    world::WorldServer, UniverseServer,
};
use aw_core::{AWPacket, VarID};

#[derive(Debug)]
enum WorldServerStartParamsError {
    Version,
    Build,
    Port,
}

struct WorldServerStartParams {
    version: u32,
    build: u32,
    port: u16,
}

impl TryFrom<&AWPacket> for WorldServerStartParams {
    type Error = WorldServerStartParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let version = value
            .get_uint(VarID::BrowserVersion)
            .ok_or(WorldServerStartParamsError::Version)?;
        let build = value
            .get_uint(VarID::WorldBuild)
            .ok_or(WorldServerStartParamsError::Build)?;
        let port = value
            .get_uint(VarID::WorldPort)
            .and_then(|port| u16::try_from(port).ok())
            .ok_or(WorldServerStartParamsError::Port)?;

        Ok(Self {
            version,
            build,
            port,
        })
    }
}

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

    let params = match WorldServerStartParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete world server start: {why:?}");
            return;
        }
    };

    conn.client = Some(ClientInfo::WorldServer(WorldServer {
        build: params.build,
        server_port: params.port,
        worlds: Vec::new(),
    }));

    log::info!(
        "Connection {} has made itself a world server. Version: 0x{:X}; Build: {}; Port {}",
        conn.addr().ip(),
        params.version,
        params.build,
        params.port,
    );
}
