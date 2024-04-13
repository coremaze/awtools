use crate::{
    client::ClientInfo,
    get_conn,
    tabs::{regenerate_contact_list_and_mutuals, regenerate_player_list},
    universe_connection::UniverseConnectionID,
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

#[derive(Debug)]
enum IdentifyParamsError {
    WorldName,
    Nonce,
    SessionID,
    PlayerIP,
    PlayerPort,
}

struct IdentifyParams {
    world_name: String,
    nonce: Vec<u8>,
    session_id: u16,
    player_ip: u32,
    player_port: u16,
}

impl TryFrom<&AWPacket> for IdentifyParams {
    type Error = IdentifyParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let world_name = value
            .get_string(VarID::WorldName)
            .ok_or(IdentifyParamsError::WorldName)?;
        let nonce = value
            .get_data(VarID::WorldUserNonce)
            .ok_or(IdentifyParamsError::Nonce)?;
        let session_id = value
            .get_uint(VarID::SessionID)
            .and_then(|sid| u16::try_from(sid).ok())
            .ok_or(IdentifyParamsError::SessionID)?;
        let player_ip = value
            .get_uint(VarID::IdentifyUserIP)
            .ok_or(IdentifyParamsError::PlayerIP)?;
        let player_port = value
            .get_uint(VarID::PlayerPort)
            .and_then(|port| u16::try_from(port).ok())
            .ok_or(IdentifyParamsError::PlayerPort)?;

        Ok(Self {
            world_name,
            nonce,
            session_id,
            player_ip,
            player_port,
        })
    }
}

/// A connection (supposed to be a world server) wants to know information about a player.
pub fn identify(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut p = AWPacket::new(PacketType::Identify);

    let params = match IdentifyParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete identify: {why:?}");
            return;
        }
    };

    {
        let conn = get_conn!(server, cid, "identify");

        let Some(world_server) = conn.world_server() else {
            return;
        };

        if world_server.get_world(&params.world_name).is_none() {
            log::info!("Failed to identify player because the world server does not own the world");
            return;
        }
    };

    let rc = identify_player(server, &params, &mut p);

    p.add_int(VarID::ReasonCode, rc as i32);
    let conn = get_conn!(server, cid, "identify");

    conn.send(p);

    // Update the user's friends to tell them this user is in a new world
    regenerate_contact_list_and_mutuals(server, cid);
}

fn identify_player(
    server: &mut UniverseServer,
    params: &IdentifyParams,
    response: &mut AWPacket,
) -> ReasonCode {
    let Some(player_cid) = server.connections.get_by_session_id(params.session_id) else {
        return ReasonCode::NoSuchSession;
    };

    let Some(user_conn) = server.connections.get_connection_mut(player_cid) else {
        log::error!("identify_player was given an invalid CID");
        return ReasonCode::NoSuchSession;
    };

    let Some(client) = &mut user_conn.client else {
        return ReasonCode::NoSuchSession;
    };

    let effective_privilege = client.effective_privilege();

    let ClientInfo::Player(player) = client else {
        return ReasonCode::NoSuchSession;
    };

    let Some(player_nonce) = player.base_player().nonce else {
        return ReasonCode::NoSuchSession;
    };

    if player_nonce != params.nonce.as_ref() {
        log::warn!("World tried to identify a player but did not have the correct nonce");
        return ReasonCode::NoSuchSession;
    }

    // Not currently checking IP address or port
    response.add_string(VarID::WorldName, params.world_name.clone());
    response.add_int(VarID::SessionID, params.session_id.into());
    response.add_uint(VarID::IdentifyUserIP, params.player_ip);
    response.add_int(VarID::PlayerPort, params.player_port.into());

    response.add_uint(VarID::LoginID, player.citizen_id().unwrap_or(0));
    response.add_int(VarID::BrowserBuild, player.base_player().build);
    response.add_string(VarID::LoginUsername, player.base_player().username.clone());
    log::trace!(
        "Effective privilege of {} is {}",
        &player.base_player().username,
        effective_privilege,
    );
    // Effective privilege controls what rights the World server gives the player.
    response.add_uint(VarID::PrivilegeUserID, effective_privilege);

    player.base_player_mut().world = Some(params.world_name.clone());

    // Regenerate the player list becase of possible change in world state
    for cid in server.connections.cids() {
        regenerate_player_list(server, cid);
    }

    ReasonCode::Success
}
