use std::net::IpAddr;

use crate::tabs::Tabs;

#[derive(Debug)]
pub enum Player {
    Citizen(Citizen),
    Tourist(GenericPlayer),
    Bot(Bot),
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

#[derive(Debug)]
pub struct Citizen {
    pub cit_id: u32,
    pub base_player: GenericPlayer,
}

#[derive(Debug)]
pub struct Bot {
    pub owner_id: u32,
    pub application: String,
    pub base_player: GenericPlayer,
}

impl Player {
    pub fn base_player(&self) -> &GenericPlayer {
        match self {
            Player::Citizen(citizen) => &citizen.base_player,
            Player::Tourist(info) => info,
            Player::Bot(bot) => &bot.base_player,
        }
    }

    pub fn base_player_mut(&mut self) -> &mut GenericPlayer {
        match self {
            Player::Citizen(citizen) => &mut citizen.base_player,
            Player::Tourist(info) => info,
            Player::Bot(bot) => &mut bot.base_player,
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
        self.base_player().username.clone()
    }
}
