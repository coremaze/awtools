use crate::{
    get_conn_mut, tabs::regenerate_contact_list, universe_connection::UniverseConnectionID,
    UniverseServer,
};
use aw_core::*;

pub fn contact_list(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let Some(starting_id) = packet.get_uint(VarID::ContactListCitizenID) else {
        return;
    };

    regenerate_contact_list(server, cid);

    let conn = get_conn_mut!(server, cid, "contact_list");

    let ip = conn.addr().ip();

    let Some(player) = conn.player_info() else {
        return;
    };

    let name = player.username.clone();

    let current_list = player.tabs.contact_list.current_starting_from(starting_id);

    log::debug!(
        "Sending the CURRENT contact list starting from id {starting_id} to {} ({}) current: {:?}",
        ip,
        name,
        current_list
    );

    current_list.send_limited_list(conn);
}
