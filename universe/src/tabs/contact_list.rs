use std::collections::HashMap;

use aw_core::{AWPacket, AWPacketGroup, PacketType, VarID};

use crate::{
    client::{ClientInfo, UniverseConnectionID},
    database::{
        contact::{ContactOptions, ContactQuery},
        CitizenDB, ContactDB,
    },
    get_conn_mut, UniverseConnection, UniverseServer,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContactState {
    Offline = 0,
    Online = 1,
    Hidden = 2,
    Afk = 3,
    Unknown = 4,
    Removed = 5,
    Default = 6,
}

/// A player in the contact list.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactListEntry {
    pub username: String,
    pub world: Option<String>,
    pub state: ContactState,
    pub citizen_id: u32,
    pub options: ContactOptions,
}

impl ContactListEntry {
    pub fn logoff(&mut self) {
        self.state = ContactState::Offline;
        self.world = None;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContactList {
    entries: HashMap<u32, ContactListEntry>,
}

impl ContactList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_contact(&mut self, contact: ContactListEntry) {
        self.entries.insert(contact.citizen_id, contact);
    }

    pub fn get_by_name(&self, name: impl AsRef<str>) -> Option<&ContactListEntry> {
        let result = self
            .entries
            .iter()
            .find(|(_, e)| e.username == name.as_ref());

        match result {
            Some((_id, e)) => Some(e),
            None => None,
        }
    }

    pub fn get_by_name_mut(&mut self, name: impl AsRef<str>) -> Option<&mut ContactListEntry> {
        let result = self
            .entries
            .iter_mut()
            .find(|(_, e)| e.username == name.as_ref());

        match result {
            Some((_id, e)) => Some(e),
            None => None,
        }
    }

    pub fn get_by_citizen_id(&self, id: u32) -> Option<&ContactListEntry> {
        self.entries.get(&id)
    }

    pub fn get_by_citizen_id_mut(&mut self, id: u32) -> Option<&mut ContactListEntry> {
        self.entries.get_mut(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn make_packet_groups(&self) -> Vec<AWPacketGroup> {
        let mut groups = Vec::<AWPacketGroup>::new();
        let mut group = AWPacketGroup::new();

        for entry in self.entries.values() {
            let mut response = AWPacket::new(PacketType::ContactList);
            response.add_string(VarID::ContactListName, entry.username.clone());
            if let Some(world) = &entry.world {
                response.add_string(VarID::ContactListWorld, world.clone());
            }
            response.add_int(VarID::ContactListStatus, entry.state as i32);
            response.add_uint(VarID::ContactListCitizenID, entry.citizen_id);
            response.add_byte(VarID::ContactListMore, 1);
            response.add_uint(VarID::ContactListOptions, entry.options.bits());

            if let Err(p) = group.push(response) {
                groups.push(group);
                group = AWPacketGroup::new();
                group.push(p).ok();
            }
        }

        let mut response = AWPacket::new(PacketType::ContactList);
        response.add_uint(VarID::ContactListCitizenID, 0);
        response.add_byte(VarID::ContactListMore, 0);

        if let Err(p) = group.push(response) {
            groups.push(group);
            group = AWPacketGroup::new();
            group.push(p).ok();
        }

        groups.push(group);

        groups
    }

    pub fn send_list(&self, target: &UniverseConnection) {
        for group in self.make_packet_groups() {
            target.send_group(group)
        }
    }
}

#[derive(Debug)]
pub struct UpdatingContactList {
    current: ContactList,
    previous: ContactList,
}

impl UpdatingContactList {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_contact(&mut self, contact: ContactListEntry) {
        self.current.add_contact(contact)
    }

    /// Returns a new ContactList containing the entries which are different, added, or removed between previous and current.
    pub fn make_difference_list(&self) -> ContactList {
        let mut list = ContactList::new();

        for entry in self.current.entries.values() {
            // If the entry is in the previous list, and it's different, add it to the difference list.
            if let Some(previous_entry) = self.previous.get_by_citizen_id(entry.citizen_id) {
                // If the entry is different, add it to the difference list.
                if previous_entry != entry {
                    list.add_contact(entry.clone());
                }
            // If the entry is not in the previous list, add it to the difference list.
            } else {
                list.add_contact(entry.clone());
            }
        }

        // If the entry is not in the current list but is in the previous list, add it to the difference list.
        // for entry in &self.previous.entries {
        //     if self.current.get_by_citizen_id(entry.citizen_id).is_none() {
        //         list.add_contact(entry.clone());
        //     }
        // }

        list
    }

    pub fn update(&mut self) {
        self.previous = self.current.clone();

        // We no longer need to track contacts who have been removed after we
        // have sent ContactState::Removed to the client.
        self.current
            .entries
            .retain(|_id, e| e.state != ContactState::Removed);
    }

    pub fn current(&self) -> &ContactList {
        &self.current
    }

    fn hide_current(&mut self) {
        for contact in self.current.entries.values_mut() {
            contact.state = ContactState::Hidden;
        }
    }
}

impl Default for UpdatingContactList {
    fn default() -> Self {
        Self {
            current: ContactList::new(),
            previous: ContactList::new(),
        }
    }
}

pub fn regenerate_contact_list(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn_mut!(server, cid, "regenerate_contact_list");
    let Some(ClientInfo::Player(player)) = &mut conn.client else {
        return;
    };
    let Some(citizen_id) = player.citizen_id() else {
        return;
    };
    let contacts = server.database.contact_get_all(citizen_id);
    let mut entries = Vec::<ContactListEntry>::new();
    for contact in &contacts {
        entries.push(contact_entry(contact, server));
    }

    let conn = get_conn_mut!(server, cid, "regenerate_contact_list");
    let Some(ClientInfo::Player(player)) = &mut conn.client else {
        return;
    };
    let contact_list = &mut player.player_info_mut().tabs.contact_list;
    contact_list.hide_current();
    for e in entries {
        contact_list.add_contact(e);
    }
}

pub fn regenerate_contact_list_and_mutuals(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn_mut!(server, cid, "regenerate_contact_list_and_mutuals");
    let Some(ClientInfo::Player(player)) = &mut conn.client else {
        return;
    };
    let Some(citizen_id) = player.citizen_id() else {
        return;
    };
    let contacts = server.database.contact_get_all(citizen_id);

    let mut cids_to_regen = vec![cid];
    for contact in contacts {
        if let Some(contact_id) = server.connections.get_by_citizen_id(contact.contact) {
            cids_to_regen.push(contact_id);
        }
    }

    for cid in cids_to_regen {
        regenerate_contact_list(server, cid);
    }
}

fn contact_entry(contact: &ContactQuery, server: &UniverseServer) -> ContactListEntry {
    let mut username = "".to_string();
    let mut world: Option<String> = None;

    let contact_citizen = match server.database.citizen_by_number(contact.contact) {
        Ok(x) => x,
        Err(_) => {
            return ContactListEntry {
                username,
                world,
                state: ContactState::Hidden,
                citizen_id: contact.contact,
                options: ContactOptions::default(),
            }
        }
    };

    username = contact_citizen.name;

    let mut status = match server.connections.get_by_citizen_id(contact.contact) {
        Some(cid) => match server.connections.get_connection(cid) {
            Some(conn) => match &conn.client {
                Some(ClientInfo::Player(p)) => {
                    world = p.player_info().world.clone();
                    if p.player_info().afk {
                        ContactState::Afk
                    } else {
                        ContactState::Online
                    }
                }
                _ => {
                    log::error!(
                        "Connection received in contact_name_world_state is not a citizen."
                    );
                    ContactState::Offline
                }
            },
            None => {
                log::error!("Got an invalid CID in contact_name_world_state");
                ContactState::Offline
            }
        },
        None => ContactState::Offline,
    };

    if !server
        .database
        .contact_status_allowed(contact.contact, contact.citizen)
    {
        status = ContactState::Unknown;
    }

    ContactListEntry {
        username,
        world,
        state: status,
        citizen_id: contact.contact,
        options: contact.options,
    }
}
