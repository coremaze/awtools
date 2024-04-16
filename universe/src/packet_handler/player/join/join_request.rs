use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;

use crate::{
    database::ContactDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};

fn send_join_reply_err(server: &UniverseServer, cid: UniverseConnectionID, err: ReasonCode) {
    let target_conn = get_conn!(server, cid, "send_join_reply_err");
    let mut response = AWPacket::new(PacketType::JoinReply);
    response.add_int(VarID::ReasonCode, err.into());
    target_conn.send(response);
}

pub fn join_request(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    // let response = AWPacket::new(PacketType::JoinReply);
    let source_conn = get_conn!(server, cid, "join_request");
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

    let they_allow_joins = match server
        .database
        .contact_joins_allowed(target_id, source_citizen_id)
    {
        DatabaseResult::Ok(allowed) => allowed,
        DatabaseResult::DatabaseError => {
            send_join_reply_err(server, cid, ReasonCode::DatabaseError);
            return;
        }
    };

    let they_allow_status = match server
        .database
        .contact_status_allowed(target_id, source_citizen_id)
    {
        DatabaseResult::Ok(allowed) => allowed,
        DatabaseResult::DatabaseError => {
            send_join_reply_err(server, cid, ReasonCode::DatabaseError);
            return;
        }
    };

    if they_allow_joins {
        if they_allow_status {
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
    let mut packet = AWPacket::new(PacketType::JoinRequest);
    packet.add_uint(VarID::CitizenNumber, source_citizen);
    packet.add_string(VarID::CitizenName, source_username);

    target_conn.send(packet);
}
