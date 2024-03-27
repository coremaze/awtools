use crate::{
    client::{ClientInfo, UniverseConnectionID},
    get_conn,
    tabs::regenerate_contact_list_and_mutuals,
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

/// A connection (supposed to be a world server) wants to know information about a player.
pub fn identify(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut p = AWPacket::new(PacketType::Identify);

    let Some(fields) = get_identify_fields(packet) else {
        return;
    };

    {
        let conn = get_conn!(server, cid, "identify");

        let Some(ClientInfo::WorldServer(w)) = &conn.client else {
            return;
        };

        if w.get_world(&fields.world_name).is_none() {
            log::info!("Failed to identify player because the world server does not own the world");
            return;
        }
    };

    let rc = identify_player(server, &fields, &mut p);

    p.add_int(VarID::ReasonCode, rc as i32);
    let conn = get_conn!(server, cid, "identify");

    conn.send(p);

    // Update the user's friends to tell them this user is in a new world
    regenerate_contact_list_and_mutuals(server, cid);
}

struct IdentifyFields {
    world_name: String,
    nonce: Vec<u8>,
    session_id: i32,
    player_ip: u32,
    player_port: i32,
}

fn get_identify_fields(packet: &AWPacket) -> Option<IdentifyFields> {
    let Some(world_name) = packet.get_string(VarID::WorldName) else {
        log::info!("Failed to identify player because no world name was provided");
        return None;
    };

    let Some(nonce) = packet.get_data(VarID::WorldUserNonce) else {
        log::info!("Failed to identify player because no user nonce was provided");
        return None;
    };

    let Some(session_id) = packet.get_int(VarID::SessionID) else {
        log::info!("Failed to identify player because no session id was provided");
        return None;
    };

    let Some(player_ip) = packet.get_uint(VarID::IdentifyUserIP) else {
        log::info!("Failed to identify player because no user ip was provided");
        return None;
    };

    let Some(player_port) = packet.get_int(VarID::PlayerPort) else {
        log::info!("Failed to identify player because no port was provided");
        return None;
    };

    Some(IdentifyFields {
        world_name,
        nonce,
        session_id,
        player_ip,
        player_port,
    })
}

fn identify_player(
    server: &mut UniverseServer,
    fields: &IdentifyFields,
    response: &mut AWPacket,
) -> ReasonCode {
    let Some(player_cid) = server
        .connections
        .get_by_session_id(fields.session_id as u16)
    else {
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

    let Some(player_nonce) = player.player_info().nonce else {
        return ReasonCode::NoSuchSession;
    };

    if player_nonce != fields.nonce.as_ref() {
        log::warn!("World tried to identify a player but did not have the correct nonce");
        return ReasonCode::NoSuchSession;
    }

    // Not currently checking IP address or port
    response.add_string(VarID::WorldName, fields.world_name.clone());
    response.add_int(VarID::SessionID, fields.session_id);
    response.add_uint(VarID::IdentifyUserIP, fields.player_ip);
    response.add_int(VarID::PlayerPort, fields.player_port);

    response.add_uint(VarID::LoginID, player.citizen_id().unwrap_or(0));
    response.add_int(VarID::BrowserBuild, player.player_info().build);
    response.add_string(VarID::LoginUsername, player.player_info().username.clone());
    log::trace!(
        "Effective privilege of {} is {}",
        &player.player_info().username,
        effective_privilege,
    );
    // Effective privilege controls what rights the World server gives the player.
    response.add_uint(VarID::PrivilegeUserID, effective_privilege);

    player.player_info_mut().world = Some(fields.world_name.clone());

    ReasonCode::Success
}
