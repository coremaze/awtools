use std::{
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    tabs::{Tabs, WorldListEntry, WorldStatus},
    world::{World, WorldServer},
    AWConnection, AWCryptRSA,
};
use aw_core::{AWPacket, AWPacketGroup, PacketType, ProtocolMessage};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerState {
    Hidden = 0,
    Online = 1,
}

/// Information that every player has.
#[derive(Debug)]
pub struct GenericPlayer {
    /// Browser build number
    pub build: i32,

    pub session_id: u16,
    pub privilege_id: Option<u32>,
    pub username: String,
    pub nonce: Option<[u8; 255]>, // AW4 worlds allow 256 bytes, AW5 worlds allow 255 bytes
    pub world: Option<String>,
    pub ip: IpAddr,
    pub afk: bool,

    pub tabs: Tabs,
}

impl GenericPlayer {
    pub fn new(session_id: u16, build: i32, privilege_id: Option<u32>, username: &str) -> Self {
        Self {
            build,
            session_id,
            privilege_id,
            username: username.to_string(),
            nonce: None,
            world: None,
            ip: IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            afk: false,
            tabs: Tabs::new(),
        }
    }
}

#[derive(Debug)]
pub struct Citizen {
    pub cit_id: u32,
    pub player_info: GenericPlayer,
}

#[derive(Debug)]
pub struct Bot {
    pub owner_id: u32,
    pub application: String,
    pub player_info: GenericPlayer,
}

#[derive(Debug)]
pub enum Player {
    Citizen(Citizen),
    Tourist(GenericPlayer),
    Bot(Bot),
}

impl Player {
    // pub fn new_citizen(
    //     citizen_id: u32,
    //     privilege_id: Option<u32>,
    //     session_id: u16,
    //     build: i32,
    //     username: &str,
    //     ip: IpAddr,
    // ) -> Self {
    //     Self::Citizen(Citizen {
    //         cit_id: citizen_id,
    //         player_info: GenericPlayer {
    //             build,
    //             session_id,
    //             privilege_id,
    //             username: username.to_string(),
    //             nonce: None,
    //             world: None,
    //             ip,
    //             afk: false,
    //             tabs: Default::default(),
    //         },
    //     })
    // }

    pub fn new_tourist(session_id: u16, build: i32, username: &str, ip: IpAddr) -> Self {
        Self::Tourist(GenericPlayer {
            build,
            session_id,
            privilege_id: None,
            username: username.to_string(),
            nonce: None,
            world: None,
            ip,
            afk: false,
            tabs: Default::default(),
        })
    }

    // pub fn new_bot(session_id: u16, username: &str, ip: IpAddr) -> Self {
    //     Self::Bot(GenericPlayer {
    //         build: 1,
    //         session_id,
    //         privilege_id: Some(1),
    //         username: username.to_string(),
    //         nonce: None,
    //         world: None,
    //         ip,
    //         afk: false,
    //         tabs: Default::default(),
    //     })
    // }

    pub fn player_info(&self) -> &GenericPlayer {
        match self {
            Player::Citizen(citizen) => &citizen.player_info,
            Player::Tourist(info) => info,
            Player::Bot(bot) => &bot.player_info,
        }
    }

    pub fn player_info_mut(&mut self) -> &mut GenericPlayer {
        match self {
            Player::Citizen(citizen) => &mut citizen.player_info,
            Player::Tourist(info) => info,
            Player::Bot(bot) => &mut bot.player_info,
        }
    }

    pub fn citizen(&self) -> Option<&Citizen> {
        match self {
            Player::Citizen(citizen) => Some(citizen),
            _ => None,
        }
    }

    pub fn citizen_mut(&mut self) -> Option<&mut Citizen> {
        match self {
            Player::Citizen(citizen) => Some(citizen),
            _ => None,
        }
    }

    pub fn citizen_id(&self) -> Option<u32> {
        self.citizen().map(|citizen| citizen.cit_id)
    }

    pub fn username(&self) -> String {
        self.player_info().username.clone()
    }
}

/// Game-related client state. Describes every client, regardless of whether
/// they are a world server, a citizen, or a tourist.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum ClientInfo {
    WorldServer(WorldServer),
    Player(Player),
}

impl ClientInfo {
    pub fn citizen(&self) -> Option<&Citizen> {
        match self {
            ClientInfo::Player(player) => player.citizen(),
            _ => None,
        }
    }

    pub fn citizen_mut(&mut self) -> Option<&mut Citizen> {
        match self {
            ClientInfo::Player(player) => player.citizen_mut(),
            ClientInfo::WorldServer(_) => None,
        }
    }

    pub fn tourist(&self) -> Option<&GenericPlayer> {
        match self {
            ClientInfo::Player(Player::Tourist(info)) => Some(info),
            _ => None,
        }
    }

    pub fn tourist_mut(&mut self) -> Option<&mut GenericPlayer> {
        match self {
            ClientInfo::Player(Player::Tourist(info)) => Some(info),
            _ => None,
        }
    }

    pub fn player(&self) -> Option<&Player> {
        match self {
            ClientInfo::WorldServer(_) => None,
            ClientInfo::Player(player) => Some(player),
        }
    }

    pub fn player_info(&self) -> Option<&GenericPlayer> {
        match self {
            ClientInfo::WorldServer(_) => None,
            ClientInfo::Player(player) => Some(player.player_info()),
        }
    }

    pub fn player_info_mut(&mut self) -> Option<&mut GenericPlayer> {
        match self {
            ClientInfo::WorldServer(_) => None,
            ClientInfo::Player(player) => Some(player.player_info_mut()),
        }
    }

    pub fn has_admin_permissions(&self) -> bool {
        // The admin account is always citizen ID 1
        if let Self::Player(Player::Citizen(citizen)) = self {
            if citizen.cit_id == 1 {
                return true;
            }
        } else if let Some(player_info) = self.player_info() {
            if player_info.privilege_id == Some(1) {
                return true;
            }
        }
        false
    }

    pub fn citizen_id(&self) -> Option<u32> {
        match self {
            Self::Player(Player::Citizen(citizen)) => Some(citizen.cit_id),
            _ => None,
        }
    }

    pub fn effective_privilege(&self) -> u32 {
        self.player_info()
            .and_then(|player| {
                player.privilege_id.and_then(|priv_id| {
                    if priv_id != 0 {
                        Some(priv_id)
                    } else {
                        self.citizen_id()
                    }
                })
            })
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct UniverseConnection {
    connection: AWConnection,
    pub rsa: AWCryptRSA,
    last_heartbeat: u64,
    /// A connection may not have one of these yet if they just connected.
    pub client: Option<ClientInfo>,
}

impl UniverseConnection {
    pub fn new(connection: AWConnection) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        Self {
            connection,
            rsa: AWCryptRSA::new(),
            last_heartbeat: now,
            client: None,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.connection.is_disconnected()
    }

    pub fn send(&self, packet: AWPacket) {
        log::trace!("Sending {} packet {packet:?}", self.addr());
        self.connection.send(packet)
    }

    pub fn send_group(&self, packets: AWPacketGroup) {
        log::trace!("Sending {} packets {packets:?}", self.addr());
        self.connection.send_group(packets)
    }

    pub fn recv(&self) -> Vec<ProtocolMessage> {
        self.connection.recv()
    }

    pub fn addr(&self) -> SocketAddr {
        self.connection.addr()
    }

    pub fn disconnect(&mut self) {
        self.connection.disconnect()
    }

    pub fn set_recv_key(&self, key: &[u8]) {
        self.connection.set_recv_key(key)
    }

    pub fn get_send_key(&self) -> Vec<u8> {
        self.connection.get_send_key()
    }

    pub fn encrypt_data(&self, should: bool) {
        self.connection.encrypt_data(should)
    }

    pub fn has_admin_permissions(&self) -> bool {
        if let Some(info) = &self.client {
            info.has_admin_permissions()
        } else {
            false
        }
    }

    pub fn player_info(&self) -> Option<&GenericPlayer> {
        if let Some(info) = &self.client {
            info.player_info()
        } else {
            None
        }
    }

    pub fn player_info_mut(&mut self) -> Option<&mut GenericPlayer> {
        if let Some(info) = &mut self.client {
            info.player_info_mut()
        } else {
            None
        }
    }

    pub fn is_player(&self) -> bool {
        self.player_info().is_some()
    }

    pub fn is_bot(&self) -> bool {
        matches!(&self.client, Some(ClientInfo::Player(Player::Bot(_))))
    }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct UniverseConnectionID(u128);

#[macro_export]
macro_rules! get_conn {
    ($server:expr, $cid:expr, $func_name:expr) => {
        match $server.connections.get_connection($cid) {
            Some(value) => value,
            None => {
                log::error!("{} was given an invalid CID.", $func_name);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! get_conn_mut {
    ($server:expr, $cid:expr, $func_name:expr) => {
        match $server.connections.get_connection_mut($cid) {
            Some(value) => value,
            None => {
                log::error!("{} was given an invalid CID.", $func_name);
                return;
            }
        }
    };
}

pub struct UniverseConnections {
    connections: HashMap<UniverseConnectionID, UniverseConnection>,
    next_id: UniverseConnectionID,
}

impl Default for UniverseConnections {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            next_id: UniverseConnectionID(0),
        }
    }
}

impl UniverseConnections {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(
        &self,
    ) -> std::collections::hash_map::Iter<'_, UniverseConnectionID, UniverseConnection> {
        // Callers shouldn't have mutable access to self.connections directly, to prevent insertions with an invalid ID
        self.connections.iter()
    }

    pub fn iter_mut(
        &mut self,
    ) -> std::collections::hash_map::IterMut<'_, UniverseConnectionID, UniverseConnection> {
        // Callers shouldn't have mutable access to self.connections directly, to prevent insertions with an invalid ID
        self.connections.iter_mut()
    }

    pub fn cids(&self) -> Vec<UniverseConnectionID> {
        return self
            .connections
            .keys()
            .copied()
            .collect::<Vec<UniverseConnectionID>>();
    }

    pub fn create_session_id(&self) -> u16 {
        let mut new_session_id: u16 = 0;
        loop {
            new_session_id = new_session_id
                .checked_add(1)
                .expect("Ran out of session IDs.");
            if self.get_by_session_id(new_session_id).is_none() {
                break;
            }
        }
        new_session_id
    }

    /// Looks up a UniverseConnectionID with a session ID if one exists
    pub fn get_by_session_id(&self, session_id: u16) -> Option<UniverseConnectionID> {
        for (&id, client) in self.iter() {
            let Some(user_info) = &client.client else {
                continue;
            };
            let Some(player_info) = user_info.player_info() else {
                continue;
            };

            if player_info.session_id == session_id {
                return Some(id);
            }
        }
        None
    }

    pub fn get_by_citizen_id(&self, citizen_id: u32) -> Option<UniverseConnectionID> {
        for (&id, conn) in self.iter() {
            if let Some(ClientInfo::Player(Player::Citizen(citizen))) = &conn.client {
                if citizen.cit_id == citizen_id {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn get_connection(&self, id: UniverseConnectionID) -> Option<&UniverseConnection> {
        self.connections.get(&id)
    }

    pub fn get_connection_mut(
        &mut self,
        id: UniverseConnectionID,
    ) -> Option<&mut UniverseConnection> {
        self.connections.get_mut(&id)
    }

    pub fn add_connection(&mut self, conn: UniverseConnection) {
        let id = self.next_id;
        self.next_id.0 = self
            .next_id
            .0
            .checked_add(1)
            .expect("Ran out of connection IDs.");
        self.connections.insert(id, conn);
    }

    pub fn send_heartbeats(&mut self) {
        for conn in self.connections.values_mut() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current time is before the unix epoch.")
                .as_secs();

            // 30 seconds between each heartbeat
            let next_heartbeat = conn.last_heartbeat + 30;

            if next_heartbeat <= now {
                log::debug!("Sending heartbeat to {}", conn.connection.addr().ip());
                let packet = AWPacket::new(PacketType::Heartbeat);
                conn.connection.send(packet);
                conn.last_heartbeat = now;
            }
        }
    }

    pub fn disconnected_cids(&self) -> Vec<UniverseConnectionID> {
        self.connections
            .iter()
            .filter(|(_, conn)| conn.is_disconnected())
            .map(|(cid, _)| *cid)
            .collect::<Vec<UniverseConnectionID>>()
    }

    pub fn remove_disconnected(&mut self) {
        for cid in self.disconnected_cids() {
            self.connections.remove(&cid);
        }
    }

    pub fn send_tab_updates(&mut self) {
        for (&_cid, conn) in self.connections.iter_mut() {
            let ip = conn.addr().ip();

            let Some(player) = conn.player_info_mut() else {
                continue;
            };

            let username = player.username.clone();

            let player_list_difference = player.tabs.player_list.make_difference_list();
            let contact_list_difference = player.tabs.contact_list.make_difference_list();
            let world_list_difference = player.tabs.world_list.make_difference_list();
            player.tabs.player_list.update();
            player.tabs.contact_list.update();
            player.tabs.world_list.update();

            if !player_list_difference.is_empty() {
                log::debug!(
                    "Sending a player list update to IP {} name {:?}. The update is {:?}",
                    ip,
                    username,
                    player_list_difference
                );

                player_list_difference.send_list(conn);
            }

            if !contact_list_difference.is_empty() {
                log::debug!(
                    "Sending a contact list update to IP {} name {:?}. The update is {:?}",
                    ip,
                    username,
                    contact_list_difference
                );

                contact_list_difference.send_list(conn);
            }

            if !world_list_difference.is_empty() {
                log::debug!(
                    "Sending a world list update to IP {} name {:?}. The update is {:?}",
                    ip,
                    username,
                    world_list_difference
                );

                world_list_difference.send_list(conn);
            }
        }
    }

    pub fn get_world_by_name(&self, name: &str) -> Option<&World> {
        for conn in self.connections.values() {
            let Some(user_info) = conn.client.as_ref() else {
                continue;
            };

            let ClientInfo::WorldServer(server) = user_info else {
                continue;
            };

            for world in &server.worlds {
                if world.name.eq_ignore_ascii_case(name) {
                    log::trace!(
                        "get_world_by_name({name:?}) got some {world:?} from conn {conn:?}"
                    );
                    return Some(world);
                }
            }
        }
        None
    }

    pub fn get_world_entry_by_name(&self, name: &str) -> Option<WorldListEntry> {
        for conn in self.connections.values() {
            let Some(user_info) = conn.client.as_ref() else {
                continue;
            };

            let ClientInfo::WorldServer(server) = user_info else {
                continue;
            };

            for world in &server.worlds {
                if world.name.eq_ignore_ascii_case(name) {
                    return Some(WorldListEntry {
                        name: world.name.clone(),
                        status: WorldStatus::from_free_entry(world.free_entry),
                        rating: world.rating,
                        ip: conn.addr().ip(),
                        port: server.server_port,
                        max_users: world.max_users,
                        world_size: world.world_size,
                        user_count: world.user_count,
                    });
                }
            }
        }
        None
    }

    pub fn get_all_world_entries(&self) -> Vec<WorldListEntry> {
        let mut entries = Vec::<WorldListEntry>::new();
        for conn in self.connections.values() {
            let Some(user_info) = conn.client.as_ref() else {
                continue;
            };

            let ClientInfo::WorldServer(server) = user_info else {
                continue;
            };

            for world in &server.worlds {
                entries.push(WorldListEntry {
                    name: world.name.clone(),
                    status: WorldStatus::from_free_entry(world.free_entry),
                    rating: world.rating,
                    ip: conn.addr().ip(),
                    port: server.server_port,
                    max_users: world.max_users,
                    world_size: world.world_size,
                    user_count: world.user_count,
                });
            }
        }
        entries
    }
}
