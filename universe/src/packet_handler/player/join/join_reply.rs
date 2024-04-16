use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    client::ClientInfo, get_conn, player::Player, universe_connection::UniverseConnectionID,
    UniverseServer,
};

pub fn join_reply(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let source_citizen_id = {
        let conn = get_conn!(server, cid, "join_reply");
        let Some(ClientInfo::Player(Player::Citizen(cit))) = &conn.client else {
            return;
        };
        cit.cit_id
    };

    let Some(citizen_id) = packet.get_uint(VarID::CitizenNumber) else {
        return;
    };

    let Some(reason_code) = packet.get_int(VarID::ReasonCode) else {
        return;
    };

    let Some(target_cid) = server.connections.get_by_citizen_id(citizen_id) else {
        // Should send NOT LOGGED IN
        return;
    };

    log::trace!("join_reply target_cid {target_cid:?}");

    let target_conn = get_conn!(server, target_cid, "join_reply");

    let mut response = AWPacket::new(PacketType::JoinReply);
    response.add_uint(VarID::CitizenNumber, source_citizen_id);
    if reason_code == ReasonCode::Success as i32 {
        let Some(world) = packet.get_string(VarID::WorldName) else {
            return;
        };

        let Some(north) = packet.get_int(VarID::PositionNorth) else {
            return;
        };

        let Some(height) = packet.get_int(VarID::PositionHeight) else {
            return;
        };

        let Some(west) = packet.get_int(VarID::PositionWest) else {
            return;
        };

        let Some(rotation) = packet.get_int(VarID::PositionRotation) else {
            return;
        };

        response.add_string(VarID::WorldName, world);
        response.add_int(VarID::PositionNorth, north);
        response.add_int(VarID::PositionHeight, height);
        response.add_int(VarID::PositionWest, west);
        response.add_int(VarID::PositionRotation, rotation);
        response.add_int(VarID::ReasonCode, reason_code);
    } else {
        response.add_int(VarID::ReasonCode, reason_code);
    }

    target_conn.send(response);
}
