use aw_core::{AWPacket, VarID};

use crate::{get_conn, universe_connection::UniverseConnectionID, UniverseServer};

use super::{eject_lookup_by_method, EjectionLookupMethod};

pub fn eject_next(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "eject_next");

    if !conn.has_admin_permissions() {
        log::trace!("eject_next failed because the client did not have permission");
        return;
    }

    let Some(address) = packet.get_uint(VarID::EjectionAddress) else {
        return;
    };

    eject_lookup_by_method(server, cid, address, EjectionLookupMethod::Next)
}
