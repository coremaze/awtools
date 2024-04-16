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

mod attribute_change;
pub use attribute_change::attribute_change;

mod world;
pub use world::*;

mod teleport;
pub use teleport::*;

mod botgram;
pub use botgram::botgram;

mod immigrate;
pub use immigrate::immigrate;

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

fn check_valid_name(mut name: &str, is_tourist: bool) -> Result<(), ReasonCode> {
    if is_tourist {
        // Tourist names must start and end with quotes
        if !name.starts_with('"') || !name.ends_with('"') {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Strip quotes to continue check
        let name_start = 1;
        let name_end = name.len().checked_sub(1).ok_or(ReasonCode::NameTooShort)?;
        name = name
            .get(name_start..name_end)
            .ok_or(ReasonCode::NameTooShort)?;
    }

    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if !name.chars().all(char::is_alphanumeric) {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}
