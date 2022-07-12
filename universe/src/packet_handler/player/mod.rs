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

use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    client::{Client, ClientManager},
    player::PlayerInfo,
};
use aw_core::*;

pub fn heartbeat(client: &Client) {
    log::info!("Received heartbeat from {}", client.addr.ip());
}

pub fn ip_to_num(ip: IpAddr) -> u32 {
    let mut res: u32 = 0;
    if let std::net::IpAddr::V4(v4) = ip {
        for octet in v4.octets().iter().rev() {
            res <<= 8;
            res |= *octet as u32;
        }
    }
    res
}

pub fn user_list(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
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

    PlayerInfo::send_updates_to_one(&client_manager.get_player_infos(), client);
}
