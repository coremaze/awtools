use crate::{get_conn, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::*;

use super::{send_license_lookup, WorldLicenseLookupMethod};

pub fn license_by_name(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "license_by_name");
    send_license_lookup(
        conn,
        packet,
        &server.database,
        WorldLicenseLookupMethod::Exact,
    );
}
