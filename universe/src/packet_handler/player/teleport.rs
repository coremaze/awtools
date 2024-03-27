use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    client::{ClientInfo, Player, UniverseConnectionID},
    database::ContactDB,
    get_conn, UniverseServer,
};

pub fn join(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    // let response = AWPacket::new(PacketType::JoinReply);
    let source_conn = get_conn!(server, cid, "join");
    let Some(source_client) = &source_conn.client else {
        return;
    };

    let Some(source_citizen_id) = source_client.citizen_id() else {
        return;
    };

    let Some(target_id) = packet.get_uint(VarID::CitizenNumber) else {
        return;
    };

    let Some(target_cid) = server.connections.get_by_citizen_id(target_id) else {
        send_join_reply_err(server, cid, ReasonCode::NotLoggedIn);
        return;
    };

    // It seems like it is supposed to refuse if the world is hidden?

    if server
        .database
        .contact_joins_allowed(target_id, source_citizen_id)
    {
        if server
            .database
            .contact_status_allowed(target_id, source_citizen_id)
        {
            send_join_request(server, target_cid, cid);
        } else {
            send_join_reply_err(server, cid, ReasonCode::NotLoggedIn);
        }
    } else {
        send_join_reply_err(server, cid, ReasonCode::JoinRefused);
    }
}

fn send_join_request(
    server: &UniverseServer,
    target_cid: UniverseConnectionID,
    source_cid: UniverseConnectionID,
) {
    let source_conn = get_conn!(server, source_cid, "send_join_request");
    let Some(source_client) = &source_conn.client else {
        return;
    };
    let Some(player_info) = source_client.player_info() else {
        return;
    };
    let source_username = player_info.username.clone();
    let Some(source_citizen) = source_client.citizen_id() else {
        return;
    };

    let target_conn = get_conn!(server, target_cid, "send_join_request");
    let mut packet = AWPacket::new(PacketType::Join);
    packet.add_uint(VarID::CitizenNumber, source_citizen);
    packet.add_string(VarID::CitizenName, source_username);

    target_conn.send(packet);
}

fn send_join_reply_err(server: &UniverseServer, cid: UniverseConnectionID, err: ReasonCode) {
    let target_conn = get_conn!(server, cid, "send_join_reply_err");
    let mut response = AWPacket::new(PacketType::JoinReply);
    response.add_int(VarID::ReasonCode, err.into());
    target_conn.send(response);
}

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
