use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    database::CitizenDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};

pub fn citizen_delete(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "citizen_lookup_by_name");

    let Some(citizen_id) = packet.get_uint(VarID::CitizenNumber) else {
        return;
    };

    let mut response = AWPacket::new(PacketType::CitizenChangeResult);

    let rc = match server.database.citizen_delete(citizen_id) {
        aw_db::DatabaseResult::Ok(()) => ReasonCode::Success,
        aw_db::DatabaseResult::DatabaseError => ReasonCode::UnableToDeleteCitizen,
    };

    response.add_int(VarID::ReasonCode, rc.into());

    conn.send(response);
}
