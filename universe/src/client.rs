use crate::{
    player::{Citizen, GenericPlayer, Player},
    world::WorldServer,
};

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
            ClientInfo::Player(player) => Some(player.base_player()),
        }
    }

    pub fn player_info_mut(&mut self) -> Option<&mut GenericPlayer> {
        match self {
            ClientInfo::WorldServer(_) => None,
            ClientInfo::Player(player) => Some(player.base_player_mut()),
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

    pub fn world_server(&self) -> Option<&WorldServer> {
        match self {
            ClientInfo::WorldServer(world_server) => Some(world_server),
            ClientInfo::Player(_) => None,
        }
    }

    pub fn world_server_mut(&mut self) -> Option<&mut WorldServer> {
        match self {
            ClientInfo::WorldServer(world_server) => Some(world_server),
            ClientInfo::Player(_) => None,
        }
    }
}
