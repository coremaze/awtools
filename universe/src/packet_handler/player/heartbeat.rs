use std::time::Instant;

use crate::{get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};

pub fn heartbeat(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn_mut!(server, cid, "heartbeat");

    log::debug!("Received heartbeat from {}", conn.addr().ip());

    conn.last_heartbeat_received = Instant::now();
}
