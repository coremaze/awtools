use std::net::IpAddr;

use crate::{get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
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
