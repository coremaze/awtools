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

use crate::{get_conn, get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::*;

pub fn heartbeat(server: &UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn!(server, cid, "heartbeat");

    log::debug!("Received heartbeat from {}", conn.addr().ip());
}

pub fn user_list(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    // This is normally based on the time, but it seems easier to just use the IDs we already have.
    let continuation_id = packet.get_uint(VarID::UserListContinuationID).unwrap_or(0);

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

    current_list.send_list_starting_from(conn, continuation_id);
}
