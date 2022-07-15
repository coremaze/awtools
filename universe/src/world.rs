use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use aw_core::{AWPacket, AWPacketGroup, AWPacketVar, PacketType, VarID};
use num_derive::FromPrimitive;

use crate::{
    client::{ClientManager, Entity},
    Client,
};

#[derive(Debug, Clone, Copy)]
pub enum WorldStatus {
    Permitted = 1,
    NotPermitted = 2,
    Hidden = 3,
}

impl WorldStatus {
    pub fn from_free_entry(free_entry: u8) -> Self {
        if free_entry != 0 {
            WorldStatus::Permitted
        } else {
            WorldStatus::NotPermitted
        }
    }
}

#[derive(FromPrimitive, Debug, Copy, Clone)]
pub enum WorldRating {
    G = 0,
    PG = 1,
    PG13 = 2,
    R = 3,
    X = 4,
}

impl Default for WorldRating {
    fn default() -> Self {
        WorldRating::G
    }
}

#[derive(Debug, Clone)]
pub struct World {
    pub name: String,
    pub status: WorldStatus,
    pub rating: WorldRating,
    pub ip: IpAddr,
    pub port: u16,
    pub max_users: u32,
    pub world_size: u32,
    pub user_count: u32,
}

impl World {
    pub fn make_list_packet(&self) -> AWPacket {
        let mut p = AWPacket::new(PacketType::WorldList);

        p.add_string(VarID::WorldListName, self.name.clone());
        p.add_byte(VarID::WorldListStatus, self.status as u8);
        p.add_uint(VarID::WorldListUsers, self.user_count);
        p.add_byte(VarID::WorldListRating, self.rating as u8);

        p
    }

    fn make_packet_groups(worlds: &[World]) -> Vec<AWPacketGroup> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        let world_packets = worlds
            .iter()
            .map(|x| x.make_list_packet())
            .collect::<Vec<AWPacket>>();

        // Group packets into larger transmissions for efficiency
        let mut groups: Vec<AWPacketGroup> = Vec::new();
        let mut group = AWPacketGroup::new();

        for world_packet in world_packets {
            if let Err(p) = group.push(world_packet) {
                groups.push(group);
                group = AWPacketGroup::new();

                let mut more = AWPacket::new(PacketType::WorldListResult);
                // Yes, expect another WorldList packet from the server
                more.add_byte(VarID::WorldListMore, 1);
                more.add_uint(VarID::WorldList3DayUnknown, now as u32);
                group.push(more).ok();
                group.push(p).ok();
            }
        }

        // Send packet indicating that the server is done
        let mut p = AWPacket::new(PacketType::WorldListResult);
        p.add_byte(VarID::WorldListMore, 0);
        p.add_uint(VarID::WorldList3DayUnknown, now as u32);

        if let Err(p) = group.push(p) {
            groups.push(group);
            group = AWPacketGroup::new();
            group.push(p).ok();
        }

        groups.push(group);

        groups
    }

    pub fn send_updates_to_some(worlds: &[World], clients: &[Client]) {
        let groups = World::make_packet_groups(worlds);

        // Send update to target players
        for client in clients {
            if let Some(Entity::Player(_)) = client.info().entity {
                for group in &groups {
                    client.connection.send_group(group.clone());
                }
            }
        }
    }

    pub fn send_updates_to_all(worlds: &[World], client_manager: &ClientManager) {
        World::send_updates_to_some(worlds, client_manager.clients());
    }

    pub fn send_update_to_all(world: &World, client_manager: &ClientManager) {
        World::send_updates_to_all(&[world.clone()], client_manager);
    }

    pub fn send_updates_to_one(worlds: &[World], target_client: &Client) {
        let groups = World::make_packet_groups(worlds);
        for group in groups {
            target_client.connection.send_group(group.clone());
        }
    }
}

#[derive(Debug)]
pub struct WorldServerInfo {
    pub build: i32,
    pub server_port: u16,
    pub worlds: Vec<World>,
}

impl WorldServerInfo {
    pub fn get_world(&self, name: &str) -> Option<&World> {
        for w in &self.worlds {
            if w.name.eq_ignore_ascii_case(name) {
                return Some(w);
            }
        }
        None
    }

    pub fn get_world_mut(&mut self, name: &str) -> Option<&mut World> {
        for w in &mut self.worlds {
            if w.name.eq_ignore_ascii_case(name) {
                return Some(w);
            }
        }
        None
    }
}
