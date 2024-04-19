use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    database::LicenseDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};

pub fn license_delete(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "license_delete");

    if !conn.has_admin_permissions() {
        log::debug!(
            "Could not complete license delete because the client has insufficient permissions."
        );
        return;
    }

    let Some(lic_name) = packet.get_string(VarID::WorldName) else {
        return;
    };

    let mut response = AWPacket::new(PacketType::LicenseChangeResult);

    let rc = match server.database.license_delete(&lic_name) {
        aw_db::DatabaseResult::Ok(()) => ReasonCode::Success,
        aw_db::DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    };

    response.add_uint(VarID::ReasonCode, rc.into());
    conn.send(response);
}
