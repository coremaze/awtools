use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    client::{Client, ClientManager, Entity},
    world::World,
};
use aw_core::*;

use rand::Rng;

use super::ip_to_num;

pub fn world_list(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    if let Some(Entity::Player(_)) = client.info().entity {
    } else {
        return;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as i32;

    // Like with UserList, I am not sure what the purpose of this is,
    // but its function is similar
    let time_val = packet.get_int(VarID::WorldList3DayUnknown).unwrap_or(0);
    if now.saturating_sub(3) < time_val {
        return;
    }

    World::send_updates_to_one(&client_manager.get_world_infos(), client);
}

pub fn world_lookup(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let mut p = AWPacket::new(PacketType::WorldLookup);

    p.add_string(VarID::WorldStartWorldName, world_name.clone());

    match client_manager.get_world_by_name(&world_name) {
        Some(world) => {
            let mut client_info = client.info_mut();
            if let Some(Entity::Player(info)) = &mut client_info.entity {
                // Build nonce
                let mut rand_bytes = [0u8; 256];
                rand::thread_rng().fill(&mut rand_bytes);

                let mut nonce = [0u8; 255];
                nonce.copy_from_slice(&rand_bytes[0..255]);
                info.nonce = Some(nonce);

                p.add_uint(VarID::WorldAddress, ip_to_num(world.ip));
                p.add_uint(VarID::WorldPort, world.port as u32);
                p.add_uint(VarID::WorldLicenseUsers, world.max_users);
                p.add_uint(VarID::WorldLicenseRange, world.world_size);
                p.add_data(VarID::WorldUserNonce, nonce.to_vec());

                p.add_int(VarID::ReasonCode, ReasonCode::Success as i32);
            }
        }
        None => {
            p.add_int(VarID::ReasonCode, ReasonCode::NoSuchWorld as i32);
        }
    }

    client.connection.send(p);
}
