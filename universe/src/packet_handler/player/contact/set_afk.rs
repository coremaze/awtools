use crate::{
    get_conn_mut, tabs::regenerate_contact_list_and_mutuals,
    universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;

pub fn set_afk(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn_mut!(server, cid, "set_afk");

    let Some(client) = &mut conn.client else {
        return;
    };

    let Some(citizen) = client.citizen_mut() else {
        return;
    };

    let afk_status = match packet.get_uint(VarID::AFKStatus) {
        Some(x) => x,
        None => return,
    };

    let is_afk = afk_status != 0;
    citizen.base_player.afk = is_afk;
    log::debug!(
        "{:?} AFK: {:?}",
        citizen.base_player.username,
        citizen.base_player.afk
    );

    // Really only need to regenerate mutuals
    regenerate_contact_list_and_mutuals(server, cid);
}
