use std::{
    cell::{Ref, RefCell, RefMut},
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    database::{
        citizen::{CitizenDB, CitizenQuery},
        Database,
    },
    AWConnection, AWCryptRSA,
};
use aw_core::{AWPacket, PacketType, ReasonCode};
use num_derive::FromPrimitive;

/// Game-related client state
#[derive(Default)]
pub struct UserInfo {
    pub build_version: Option<i32>,
    pub session_id: Option<u16>,
    pub citizen_id: Option<u32>,
    pub username: Option<String>,
    pub client_type: Option<ClientType>,
}

pub struct Client {
    pub connection: AWConnection,
    pub dead: RefCell<bool>,
    pub rsa: AWCryptRSA,
    user_info: RefCell<UserInfo>,
    pub addr: SocketAddr,
    pub last_heartbeat: u64,
}

impl Client {
    pub fn new(connection: AWConnection, addr: SocketAddr) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        Self {
            connection,
            dead: RefCell::new(false),
            rsa: AWCryptRSA::new(),
            user_info: RefCell::new(Default::default()),
            addr,
            last_heartbeat: now,
        }
    }

    pub fn kill(&self) {
        *self.dead.borrow_mut() = true;
    }

    pub fn is_dead(&self) -> bool {
        *self.dead.borrow()
    }

    pub fn info_mut(&self) -> RefMut<UserInfo> {
        self.user_info.borrow_mut()
    }

    pub fn info(&self) -> Ref<UserInfo> {
        self.user_info.borrow()
    }
}

#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum ClientType {
    World = 1,
    UnspecifiedHuman = 2, // Temporary state between Citizen or Tourist
    Bot = 3,
    Citizen = 4,
    Tourist = 5,
}

#[derive(Default)]
pub struct ClientManager {
    clients: Vec<Client>,
}

impl ClientManager {
    pub fn create_session_id(&self) -> u16 {
        let mut new_session_id: u16 = 0;
        while new_session_id == 0 {
            new_session_id += 1;
            if self.get_client_by_session_id(new_session_id).is_none() {
                break;
            }
        }
        new_session_id
    }

    pub fn get_client_by_session_id(&self, session_id: u16) -> Option<&Client> {
        for client in &self.clients {
            if (*client.user_info.borrow()).session_id == Some(session_id) {
                return Some(client);
            }
        }
        None
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.push(client);
    }

    pub fn clients(&self) -> &Vec<Client> {
        &self.clients
    }

    pub fn remove_dead_clients(&mut self) {
        self.clients = self.clients.drain(..).filter(|x| !x.is_dead()).collect();
    }

    pub fn check_tourist(&self, username: &str) -> Result<(), ReasonCode> {
        check_valid_name(username, true)?;

        for other_client in self.clients() {
            if let Some(other_username) = &other_client.info().username {
                if other_username == username {
                    return Err(ReasonCode::NameAlreadyUsed);
                }
            }
        }
        Ok(())
    }

    pub fn check_citizen(
        &self,
        db: &Database,
        client: &Client,
        username: &Option<String>,
        password: &Option<String>,
        priv_id: Option<u32>,
        priv_pass: &Option<String>,
    ) -> Result<CitizenQuery, ReasonCode> {
        // Name and password must be present
        let password = password.as_ref().ok_or(ReasonCode::InvalidPassword)?;
        if password.is_empty() {
            return Err(ReasonCode::InvalidPassword);
        }

        let username = username.as_ref().ok_or(ReasonCode::NoSuchCitizen)?;
        if username.is_empty() {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Name cannot be bot or tourist
        if username.starts_with('[') || username.starts_with('"') {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Checks if acquiring a privilege
        if let Some(priv_id) = priv_id.filter(|x| *x != 0) {
            // Get acting citizen
            let priv_citizen = db
                .citizen_by_number(priv_id)
                .map_err(|_| ReasonCode::NoSuchActingCitizen)?;

            // Is it enabled?
            if priv_citizen.enabled == 0 && priv_citizen.id != 1 {
                return Err(ReasonCode::NoSuchActingCitizen);
            }

            // Is the priv pass present and correct?
            let priv_pass = priv_pass
                .as_ref()
                .ok_or(ReasonCode::ActingPasswordInvalid)?;
            if *priv_pass != priv_citizen.priv_pass {
                return Err(ReasonCode::ActingPasswordInvalid);
            }
        }

        // Get login citizen
        let login_citizen = db
            .citizen_by_name(username)
            .or(Err(ReasonCode::NoSuchCitizen))?;

        // Is login password correct?
        if login_citizen.password != *password {
            return Err(ReasonCode::InvalidPassword);
        }

        // Is it enabled?
        if login_citizen.enabled == 0 {
            return Err(ReasonCode::CitizenDisabled);
        }

        // Is this citizen already logged in?
        for other_client in self.clients() {
            if other_client.info().citizen_id == Some(login_citizen.id) {
                // Don't give an error if the client is already logged in as this user.
                if client as *const Client != other_client as *const Client {
                    return Err(ReasonCode::IdentityAlreadyInUse);
                }
            }
        }

        Ok(login_citizen)
    }

    pub fn send_heartbeats(&mut self) {
        for client in &mut self.clients {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current time is before the unix epoch.")
                .as_secs();

            // 30 seconds between each heartbeat
            let next_heartbeat = client.last_heartbeat + 30;

            if next_heartbeat <= now {
                log::info!("Sending heartbeat to {}", client.addr.ip());
                let packet = AWPacket::new(PacketType::Heartbeat);
                client.connection.send(packet);
                client.last_heartbeat = now;
            }
        }
    }
}

fn check_valid_name(name: &str, is_tourist: bool) -> Result<(), ReasonCode> {
    let mut name = name.to_string();

    if is_tourist {
        // Tourist names must start and end with quotes
        if !name.starts_with('"') || !name.ends_with('"') {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Strip quotes to continue check
        name.remove(0);
        name.remove(name.len() - 1);
    }

    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if !name.chars().all(char::is_alphanumeric) {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}
