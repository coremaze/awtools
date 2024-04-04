use std::{collections::HashMap, net::IpAddr};

use aw_core::{AWPacket, AWPacketGroup, PacketType, VarID};

use crate::{
    client::ClientInfo, get_conn_mut, player::Player, universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Hidden = 0,
    InWorld = 1,
    Available = 2,
}

/// A player in the player list.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerListEntry {
    pub citizen_id: Option<u32>,
    pub privilege_id: Option<u32>,
    pub username: String,
    pub world: Option<String>,
    pub ip: IpAddr,
    pub state: PlayerState,
    pub afk: bool,
}

impl PlayerListEntry {
    pub fn from_player(player: &Player) -> Self {
        Self {
            citizen_id: player.citizen_id(),
            privilege_id: player.base_player().privilege_id,
            username: player.base_player().username.clone(),
            world: player.base_player().world.clone(),
            ip: player.base_player().ip,
            // The first-party Universe uses Available only if the user is a bot.
            // However, the only person who can see bots in the user list is the Administrator,
            // who has a different user list which does not show this information.
            // As a result, "Available" is never actually shown in that universe.
            // I think this is a bug, so I am making it apply to all users, not just bots.
            // This means that users who have connected to the Universe but which have not
            // yet joined a world will be shown as "Available" instead of "In World".
            state: if player.base_player().world.is_some() {
                PlayerState::InWorld
            } else {
                PlayerState::Available
            },
            afk: player.base_player().afk,
        }
    }

    pub fn make_list_packet(&self, to_admin: bool, id_in_list: PlayerListID) -> AWPacket {
        let mut p = AWPacket::new(PacketType::UserList);

        // p.add_string(VarID::UserListEmailAddress, format!("178"));
        p.add_string(VarID::UserListName, self.username.clone());
        p.add_uint(VarID::UserListID, id_in_list.0);
        p.add_uint(VarID::UserListCitizenID, self.citizen_id.unwrap_or(0));
        p.add_uint(VarID::UserListPrivilegeID, self.privilege_id.unwrap_or(0));
        if to_admin {
            p.add_uint(VarID::UserListAddress, ip_to_num(self.ip));
        }
        p.add_byte(VarID::UserListState, self.state as u8);

        if let Some(world_name) = &self.world {
            p.add_string(VarID::UserListWorldName, world_name.clone());
        }

        p
    }
}

/// A unique identifier for a player in the player list.
#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct PlayerListID(u32);

impl PlayerListID {
    fn increment(&self) -> PlayerListID {
        // 0 should not be valid
        let next_u32 = self.0.checked_add(1).unwrap_or(1);
        PlayerListID(next_u32)
    }
}

impl Default for PlayerListID {
    fn default() -> Self {
        // 0 should not be valid
        Self(1)
    }
}

impl PartialOrd for PlayerListID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlayerListID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// The list of players that a client is currently aware of.
#[derive(Debug, Clone, Default)]
pub struct PlayerList {
    players: HashMap<PlayerListID, PlayerListEntry>,
    next_id: PlayerListID,
}

impl PlayerList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the next valid PlayerListID.
    fn get_next_valid_id(&self) -> PlayerListID {
        let mut new_id = self.next_id;
        let starting_id = new_id;
        while self.players.contains_key(&new_id) {
            new_id = new_id.increment();

            if new_id == starting_id {
                panic!("No valid player list IDs left!");
            }
        }
        new_id
    }

    /// Updates the next_id to the next valid PlayerListID.
    fn increment_next_id(&mut self) {
        self.next_id = self.next_id.increment();
    }

    /// Adds a player to the list and returns their PlayerListID.
    pub fn add_player(&mut self, player: PlayerListEntry) -> PlayerListID {
        // Make sure IDs don't change if they don't have to
        for (id, existing_player) in &mut self.players {
            if existing_player.username == player.username
                && existing_player.citizen_id == player.citizen_id
            {
                *existing_player = player;
                return *id;
            }
        }

        let id = self.get_next_valid_id();
        self.increment_next_id();

        self.players.insert(id, player);
        id
    }

    pub fn get_by_name(&mut self, name: impl AsRef<str>) -> Option<&mut PlayerListEntry> {
        self.players
            .values_mut()
            .find(|player| player.username == name.as_ref())
    }

    /// Returns the PlayerListIDs which are changed between self and other.
    pub fn difference(&self, other: &Self) -> Vec<PlayerListID> {
        let mut changed_ids = Vec::new();

        for (id, player) in &self.players {
            if let Some(other_player) = other.players.get(id) {
                // Player exists in both lists
                if player != other_player {
                    // Player was changed
                    changed_ids.push(*id);
                }
            } else {
                // Player was removed from other
                changed_ids.push(*id);
            }
        }

        for id in other.players.keys() {
            if !self.players.contains_key(id) {
                // Player was added to other
                changed_ids.push(*id);
            }
        }

        changed_ids
    }

    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    fn make_packet_group_starting_from(
        &self,
        continuation_id: u32,
        to_admin: bool,
    ) -> AWPacketGroup {
        let ids_in_order = {
            let mut k = self
                .players
                .keys()
                .filter(|id| id.0 >= continuation_id)
                .copied()
                .collect::<Vec<PlayerListID>>();
            k.sort();
            k
        };

        // Group packets into larger transmissions for efficiency
        let mut group = AWPacketGroup::new();
        let mut next_continuation_id: Option<PlayerListID> = None;

        for list_id in ids_in_order {
            let Some(player) = self.players.get(&list_id) else {
                log::warn!("Attempted to get invalid id {list_id:?} from player list {self:?}");
                continue;
            };

            if group.serialize_len() > 0x1000 {
                next_continuation_id = Some(list_id);
                break;
            }

            let entry_packet = player.make_list_packet(to_admin, list_id);
            // This generally should not fail because we are stopping way before the max size of a group
            if group.push(entry_packet).is_err() {
                log::warn!("Failed to add a packet to a player list group. (1)");
            };
        }

        let mut p = AWPacket::new(PacketType::UserListResult);
        match next_continuation_id {
            Some(next) => {
                p.add_byte(VarID::UserListMore, 1);
                p.add_uint(VarID::UserListContinuationID, next.0);
            }
            None => {
                p.add_byte(VarID::UserListMore, 0);
                p.add_uint(VarID::UserListContinuationID, 0);
            }
        }
        if group.push(p).is_err() {
            log::warn!("Failed to add a packet to a player list group. (2)");
        };

        group
    }

    pub fn send_full_list(&self, target: &UniverseConnection) {
        let mut group = AWPacketGroup::new();

        for (id, player) in &self.players {
            if group.serialize_len() > 0x4000 {
                target.send_group(group);
                group = AWPacketGroup::new();
            }
            let packet = player.make_list_packet(target.has_admin_permissions(), *id);
            group.push(packet).ok();

            let mut more = AWPacket::new(PacketType::UserListResult);
            more.add_byte(VarID::UserListMore, 0);
            more.add_uint(VarID::UserListContinuationID, id.0);
            group.push(more).ok();
        }
        target.send_group(group);
    }

    pub fn send_list_starting_from(&self, target: &UniverseConnection, continuation_id: u32) {
        let group =
            self.make_packet_group_starting_from(continuation_id, target.has_admin_permissions());

        target.send_group(group);
    }
}

/// The list of players that a client is currently aware of, tracking changes that have been made since the last update.
#[derive(Debug)]
pub struct UpdatingPlayerList {
    current: PlayerList,
    previous: PlayerList,
}

impl UpdatingPlayerList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a player to the list and returns their PlayerListID.
    fn add_player(&mut self, player: PlayerListEntry) -> PlayerListID {
        self.current.add_player(player)
    }

    /// Returns the PlayerListIDs which are changed between self and other.
    fn difference(&self) -> Vec<PlayerListID> {
        self.current.difference(&self.previous)
    }

    /// Updates the previous list to match the current list.
    pub fn update(&mut self) {
        // Remove players who are offline, we no longer need to track them.
        let ids_to_remove = self
            .current
            .players
            .iter()
            .filter(|(_, player)| player.state == PlayerState::Hidden)
            .map(|(id, _)| *id)
            .collect::<Vec<PlayerListID>>();

        for id in ids_to_remove {
            self.current.players.remove(&id);
        }

        self.previous = self.current.clone();
    }

    pub fn current(&self) -> &PlayerList {
        &self.current
    }

    /// Creates a new PlayerList from the current list, consisting only of the players that have changed since the last update.
    pub fn make_difference_list(&self) -> PlayerList {
        let mut list = PlayerList::new();

        let changed_ids = self.difference();

        for id in self
            .current
            .players
            .keys()
            .filter(|id| changed_ids.contains(id))
        {
            list.players.insert(*id, self.current.players[id].clone());
        }

        list
    }

    fn hide_current(&mut self) {
        for p in self.current.players.values_mut() {
            p.state = PlayerState::Hidden;
        }
    }
}

impl Default for UpdatingPlayerList {
    fn default() -> Self {
        Self {
            current: PlayerList::new(),
            previous: PlayerList::new(),
        }
    }
}

pub fn regenerate_player_list(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let mut entries = Vec::<PlayerListEntry>::new();
    // Add everyone to this client's player list
    for (&_cid, other_conn) in server.connections.iter() {
        let Some(other_client) = &other_conn.client else {
            continue;
        };
        let ClientInfo::Player(other_player) = &other_client else {
            continue;
        };

        let entry = PlayerListEntry::from_player(other_player);
        entries.push(entry);
    }

    let conn = get_conn_mut!(server, cid, "regenerate_player_list");
    let Some(ClientInfo::Player(p)) = &mut conn.client else {
        return;
    };
    let player_list = &mut p.base_player_mut().tabs.player_list;
    player_list.hide_current();
    for e in entries {
        player_list.add_player(e);
    }
}
