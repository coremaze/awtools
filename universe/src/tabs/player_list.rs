use std::{
    collections::HashMap,
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

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
    Online = 1,
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
            privilege_id: player.player_info().privilege_id,
            username: player.player_info().username.clone(),
            world: player.player_info().world.clone(),
            ip: player.player_info().ip,
            state: PlayerState::Online,
            afk: player.player_info().afk,
        }
    }

    pub fn make_list_packet(&self, to_admin: bool, id_in_list: PlayerListID) -> AWPacket {
        let mut p = AWPacket::new(PacketType::UserList);

        // Client also expects var 178 as a string, but don't know what it is for.
        // p.add_string(VarID::UserList178, format!("178"));
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

/// The list of players that a client is currently aware of.
#[derive(Debug, Clone)]
pub struct PlayerList {
    players: HashMap<PlayerListID, PlayerListEntry>,
    next_id: PlayerListID,
}

impl Default for PlayerList {
    fn default() -> Self {
        Self {
            players: HashMap::new(),
            next_id: PlayerListID(0),
        }
    }
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
            new_id.0 = new_id.0.wrapping_add(1);
            if new_id == starting_id {
                panic!("No valid player list IDs left!");
            }
        }
        new_id
    }

    /// Updates the next_id to the next valid PlayerListID.
    fn increment_next_id(&mut self) {
        self.next_id.0 = self.next_id.0.wrapping_add(1);
    }

    /// Adds a player to the list and returns their PlayerListID.
    pub fn add_player(&mut self, player: PlayerListEntry) -> PlayerListID {
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

    fn make_packet_groups(&self, to_admin: bool) -> Vec<AWPacketGroup> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        let player_packets = self
            .players
            .iter()
            .map(|(id, player)| player.make_list_packet(to_admin, *id))
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
                more.add_byte(VarID::UserListMore, 1);
                more.add_uint(VarID::UserList3DayUnknown, now as u32);
                group.push(more).ok();
                group.push(p).ok();
            }
        }

        // Send packet indicating that the server is done
        let mut p = AWPacket::new(PacketType::UserListResult);
        p.add_byte(VarID::UserListMore, 0);
        p.add_uint(VarID::UserList3DayUnknown, now as u32);

        if let Err(p) = group.push(p) {
            groups.push(group);
            group = AWPacketGroup::new();
            group.push(p).ok();
        }

        groups.push(group);

        groups
    }

    pub fn send_list(&self, target: &UniverseConnection) {
        let groups = self.make_packet_groups(target.has_admin_permissions());

        for group in groups {
            target.send_group(group.clone());
        }
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
    let player_list = &mut p.player_info_mut().tabs.player_list;
    player_list.hide_current();
    for e in entries {
        player_list.add_player(e);
    }
}
