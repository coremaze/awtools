use std::net::IpAddr;

use crate::{client::UniverseConnectionID, get_conn, get_conn_mut, UniverseServer};
use aw_core::*;

use rand::Rng;

fn ip_to_num(ip: IpAddr) -> u32 {
    let mut res: u32 = 0;
    if let std::net::IpAddr::V4(v4) = ip {
        for octet in v4.octets().iter().rev() {
            res <<= 8;
            res |= *octet as u32;
        }
    }
    res
}

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

pub fn world_lookup(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let world_name = match packet.get_string(VarID::WorldName) {
        Some(x) => x,
        None => return,
    };

    let mut p = AWPacket::new(PacketType::WorldLookup);

    p.add_string(VarID::WorldName, world_name.clone());

    match server.connections.get_world_entry_by_name(&world_name) {
        Some(world) => {
            let max_users = world.max_users;
            let world_size = world.world_size;
            let conn = get_conn_mut!(server, cid, "world_lookup");
            if let Some(player) = conn.player_info_mut() {
                // Build nonce
                let mut rand_bytes = [0u8; 256];
                rand::thread_rng().fill(&mut rand_bytes);

                let mut nonce = [0u8; 255];
                nonce.copy_from_slice(&rand_bytes[0..255]);
                player.nonce = Some(nonce);

                p.add_uint(VarID::WorldAddress, ip_to_num(world.ip));
                p.add_uint(VarID::WorldPort, world.port as u32);
                p.add_uint(VarID::WorldLicenseUsers, max_users);
                p.add_uint(VarID::WorldLicenseRange, world_size);
                p.add_data(VarID::WorldUserNonce, nonce.to_vec());

                p.add_int(VarID::ReasonCode, ReasonCode::Success as i32);
            }
        }
        None => {
            p.add_int(VarID::ReasonCode, ReasonCode::NoSuchWorld as i32);
        }
    }

    let conn = get_conn_mut!(server, cid, "world_lookup");
    conn.send(p);
}
