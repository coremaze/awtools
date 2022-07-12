use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use aw_core::{AWPacket, AWPacketGroup, AWPacketVar, PacketType, VarID};

use crate::{
    client::{ClientManager, Entity},
    Client,
};

#[derive(Debug, Clone, Copy)]
pub enum PlayerState {
    Offline = 0,
    Online = 1,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub build: i32,
    pub session_id: u16,
    pub citizen_id: Option<u32>,
    pub privilege_id: Option<u32>,
    pub username: String,
    pub nonce: Option<[u8; 255]>, // AW4 worlds allow 256 bytes, AW5 worlds allow 255 bytes
    pub world: Option<String>,
    pub ip: IpAddr,
    pub state: PlayerState,
    pub afk: bool,
}

impl PlayerInfo {
    pub fn effective_privilege(&self) -> u32 {
        match self.privilege_id {
            Some(id) => match id {
                0 => self.citizen_id.unwrap_or(0),
                _ => id,
            },
            None => self.citizen_id.unwrap_or(0),
        }
    }

    pub fn make_list_packet(&self, to_admin: bool) -> AWPacket {
        let mut p = AWPacket::new(PacketType::UserList);

        // Client also expects var 178 as a string, but don't know what it is for.
        // p.add_var(AWPacketVar::String(VarID::UserList178, format!("178")));
        p.add_var(AWPacketVar::String(
            VarID::UserListName,
            self.username.clone(),
        ));

        // ID is supposed to be an ID relating to the user list so it can
        // be updated when a user changes state, but using the session id
        // for this is convenient for now.
        p.add_var(AWPacketVar::Int(VarID::UserListID, self.session_id.into()));

        p.add_var(AWPacketVar::Uint(
            VarID::UserListCitizenID,
            self.citizen_id.unwrap_or(0),
        ));
        p.add_var(AWPacketVar::Uint(
            VarID::UserListPrivilegeID,
            self.privilege_id.unwrap_or(0),
        ));
        if to_admin {
            p.add_var(AWPacketVar::Uint(
                VarID::UserListAddress,
                ip_to_num(self.ip),
            ));
        }
        p.add_var(AWPacketVar::Byte(VarID::UserListState, self.state as u8));

        if let Some(world_name) = &self.world {
            p.add_var(AWPacketVar::String(
                VarID::UserListWorldName,
                world_name.clone(),
            ));
        }

        p
    }

    fn make_packet_groups(players: &[PlayerInfo], to_admin: bool) -> Vec<AWPacketGroup> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        let player_packets = players
            .iter()
            .map(|x| x.make_list_packet(to_admin))
            .collect::<Vec<AWPacket>>();

        // Group packets into larger transmissions for efficiency
        let mut groups: Vec<AWPacketGroup> = Vec::new();
        let mut group = AWPacketGroup::new();

        for player_packet in player_packets {
            if let Err(p) = group.push(player_packet) {
                groups.push(group);
                group = AWPacketGroup::new();

                let mut more = AWPacket::new(PacketType::UserListResult);
                // Yes, expect another UserList packet from the server
                more.add_var(AWPacketVar::Byte(VarID::UserListMore, 1));
                more.add_var(AWPacketVar::Uint(VarID::UserList3DayUnknown, now as u32));
                group.push(more).ok();
                group.push(p).ok();
            }
        }

        // Send packet indicating that the server is done
        let mut p = AWPacket::new(PacketType::UserListResult);
        p.add_var(AWPacketVar::Byte(VarID::UserListMore, 0));
        p.add_var(AWPacketVar::Uint(VarID::UserList3DayUnknown, now as u32));

        if let Err(p) = group.push(p) {
            groups.push(group);
            group = AWPacketGroup::new();
            group.push(p).ok();
        }

        groups.push(group);

        groups
    }

    pub fn send_updates_to_some(players: &[PlayerInfo], clients: &[Client]) {
        let groups_normal = PlayerInfo::make_packet_groups(players, false);
        let groups_admin = PlayerInfo::make_packet_groups(players, true);

        // Send update to target players
        for client in clients {
            if let Some(Entity::Player(_)) = client.info().entity {
                // Only send the groups with IP addresses to admins.
                if client.has_admin_permissions() {
                    for group in &groups_admin {
                        client.connection.send_group(group.clone());
                    }
                } else {
                    for group in &groups_normal {
                        client.connection.send_group(group.clone());
                    }
                }
            }
        }
    }

    pub fn send_updates_to_all(players: &[PlayerInfo], client_manager: &ClientManager) {
        PlayerInfo::send_updates_to_some(players, client_manager.clients());
    }

    pub fn send_update_to_all(player: &PlayerInfo, client_manager: &ClientManager) {
        PlayerInfo::send_updates_to_all(&[player.clone()], client_manager);
    }

    pub fn send_updates_to_one(players: &[PlayerInfo], target_client: &Client) {
        let groups = if target_client.has_admin_permissions() {
            PlayerInfo::make_packet_groups(players, true)
        } else {
            PlayerInfo::make_packet_groups(players, false)
        };

        for group in groups {
            target_client.connection.send_group(group.clone());
        }
    }
}

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
