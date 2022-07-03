use std::cell::{Ref, RefCell, RefMut};

use crate::{AWConnection, AWCryptRSA};
use num_derive::FromPrimitive;

#[derive(Default)]
pub struct UserInfo {
    pub build_version: Option<i32>,
    pub session_id: Option<u16>,
}

pub struct Client {
    pub connection: AWConnection,
    pub dead: RefCell<bool>,
    pub rsa: AWCryptRSA,
    user_info: RefCell<UserInfo>,
}

impl Client {
    pub fn new(connection: AWConnection) -> Self {
        Self {
            connection,
            dead: RefCell::new(false),
            rsa: AWCryptRSA::new(),
            user_info: RefCell::new(Default::default()),
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
}
