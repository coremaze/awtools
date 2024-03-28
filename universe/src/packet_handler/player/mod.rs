mod login;
pub use login::*;

mod citizen;
pub use citizen::*;

mod license;
pub use license::*;

mod contact;
pub use contact::*;

mod telegram;
pub use telegram::*;

mod attribute;
pub use attribute::*;

mod world;
pub use world::*;

mod teleport;
pub use teleport::*;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{get_conn, get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::*;

pub fn heartbeat(server: &UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn!(server, cid, "heartbeat");

    log::debug!("Received heartbeat from {}", conn.addr().ip());
}

pub fn user_list(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as i32;

    // I am not entirely sure what the purpose of this is, but it has some sort
    // of relation to 3 days. It sends our values back to us with this, so we
    // can use this to deny the client from spamming for updates, which causes
    // flickering of the user list with very large numbers of players.
    let time_val = packet.get_int(VarID::UserList3DayUnknown).unwrap_or(0);
    if now.saturating_sub(3) < time_val {
        return;
    }

    let conn = get_conn_mut!(server, cid, "user_list");

    let ip = conn.addr().ip();

    let Some(player) = conn.player_info_mut() else {
        return;
    };

    let name = player.username.clone();

    let player_list = &mut player.tabs.player_list;

    let current_list = player_list.current().clone();

    log::debug!(
        "Sending the full CURRENT player list to {} ({}) current: {:?}",
        ip,
        name,
        current_list
    );

    current_list.send_list(conn);
}
