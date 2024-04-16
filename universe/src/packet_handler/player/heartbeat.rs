use crate::{get_conn, universe_connection::UniverseConnectionID, UniverseServer};

pub fn heartbeat(server: &UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn!(server, cid, "heartbeat");

    log::debug!("Received heartbeat from {}", conn.addr().ip());
}
