use crate::{get_conn, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::*;

pub fn world_list(server: &UniverseServer, cid: UniverseConnectionID, _packet: &AWPacket) {
    let conn = get_conn!(server, cid, "world_list");

    if !conn.is_player() {
        return;
    }

    // let now = SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .expect("Current time is before the unix epoch.")
    //     .as_secs() as i32;

    // // Like with UserList, I am not sure what the purpose of this is,
    // // but its function is similar
    // let time_val = packet.get_int(VarID::WorldList3DayUnknown).unwrap_or(0);
    // if now.saturating_sub(3) < time_val {
    //     return;
    // }

    let conn = get_conn!(server, cid, "world_list");

    let ip = conn.addr().ip();

    let Some(player) = conn.player_info() else {
        return;
    };

    let name = player.username.clone();

    let world_list = &player.tabs.world_list;

    let current_list = world_list.current().clone();

    log::debug!(
        "Sending the full CURRENT world list to {} ({}) current: {:?}",
        ip,
        name,
        current_list
    );

    current_list.send_list(conn);
}
