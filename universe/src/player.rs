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

impl Player {
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
