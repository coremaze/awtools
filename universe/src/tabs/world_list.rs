use std::{
    collections::HashMap,
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use aw_core::{AWPacket, AWPacketGroup, PacketType, VarID};

use crate::{
    client::{ClientInfo, UniverseConnectionID},
    get_conn_mut,
    world::WorldRating,
    UniverseConnection, UniverseServer,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorldStatus {
    Public = 1,
    Private = 2,
    Hidden = 3,
}

impl WorldStatus {
    pub fn from_free_entry(free_entry: bool) -> Self {
        if free_entry {
            WorldStatus::Public
        } else {
            WorldStatus::Private
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldListEntry {
    pub name: String,
    pub status: WorldStatus,
    pub rating: WorldRating,
    pub ip: IpAddr,
    pub port: u16,
    pub max_users: u32,
    pub world_size: u32,
    pub user_count: u32,
}

impl WorldListEntry {
    pub fn make_list_packet(&self) -> AWPacket {
        let mut p = AWPacket::new(PacketType::WorldList);

        p.add_string(VarID::WorldListName, self.name.clone());
        p.add_byte(VarID::WorldListStatus, self.status as u8);
        p.add_uint(VarID::WorldListUsers, self.user_count);
        p.add_byte(VarID::WorldListRating, self.rating as u8);

        p
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorldList {
    entries: HashMap<String, WorldListEntry>,
}

impl WorldList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn add_world(&mut self, world: WorldListEntry) {
        self.entries.insert(world.name.clone(), world);
    }

    fn get_by_name(&self, name: &str) -> Option<&WorldListEntry> {
        self.entries.values().find(|&entry| entry.name == name)
    }

    pub fn make_packet_groups(&self) -> Vec<AWPacketGroup> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        let world_packets = self
            .entries
            .values()
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

    pub fn send_list(&self, target: &UniverseConnection) {
        let groups = self.make_packet_groups();

        for group in groups {
            target.send_group(group.clone());
        }
    }
}

/// The list of worlds that a client is currently aware of, tracking changes that have been made since the last update.
#[derive(Debug)]
pub struct UpdatingWorldList {
    current: WorldList,
    previous: WorldList,
}

impl UpdatingWorldList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) {
        // Remove worlds who are offline, we no longer need to track them.
        self.previous = self.current.clone();

        let names_to_remove = self
            .current
            .entries
            .iter()
            .filter(|(_, entry)| entry.status == WorldStatus::Hidden)
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>();

        for id in names_to_remove {
            self.current.entries.remove(&id);
        }
    }

    pub fn current(&self) -> &WorldList {
        &self.current
    }

    fn add_world(&mut self, world: WorldListEntry) {
        self.current.add_world(world)
    }

    /// Returns a new WorldList containing the entries which are different, added, or removed between previous and current.
    pub fn make_difference_list(&self) -> WorldList {
        let mut list = WorldList::new();

        for entry in self.current.entries.values() {
            // If the entry is in the previous list, and it's different, add it to the difference list.
            if let Some(previous_entry) = self.previous.get_by_name(&entry.name) {
                // If the entry is different, add it to the difference list.
                if previous_entry != entry {
                    list.add_world(entry.clone());
                }
            // If the entry is not in the previous list, add it to the difference list.
            } else {
                list.add_world(entry.clone());
            }
        }

        list
    }

    fn hide_current(&mut self) {
        for world in self.current.entries.values_mut() {
            world.status = WorldStatus::Hidden
        }
    }
}

impl Default for UpdatingWorldList {
    fn default() -> Self {
        Self {
            current: WorldList::new(),
            previous: WorldList::new(),
        }
    }
}

pub fn regenerate_world_list(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let entries = server.connections.get_all_world_entries();
    let conn = get_conn_mut!(server, cid, "regenerate_world_list");
    let Some(ClientInfo::Player(p)) = &mut conn.client else {
        return;
    };
    log::trace!("regenerate_world_list: Adding worlds to cid {cid:?}");
    let world_list = &mut p.player_info_mut().tabs.world_list;
    world_list.hide_current();
    for e in entries {
        log::trace!("regenerate_world_list: {e:?}");
        world_list.add_world(e);
    }
}
