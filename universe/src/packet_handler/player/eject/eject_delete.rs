use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;

use crate::{
    database::EjectDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};

pub fn eject_delete(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "eject_delete");

    if !conn.has_admin_permissions() {
        log::trace!("eject_delete failed because the client did not have permission");
        return;
    }

    let Some(address) = packet.get_uint(VarID::EjectionAddress) else {
        return;
    };

    let mut response = AWPacket::new(PacketType::EjectResult);

    let rc = match server.database.ejection_delete(address) {
        DatabaseResult::Ok(()) => ReasonCode::Success,
        DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    };

    response.add_uint(VarID::ReasonCode, rc.into());
    conn.send(response);
}
