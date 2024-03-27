use super::Database;
use crate::database;
use aw_core::ReasonCode;
use bitflags::bitflags;
use mysql::prelude::*;
use mysql::*;

type Result<T, E> = std::result::Result<T, E>;

bitflags! {
    #[derive(Default)]
    pub struct ContactOptions : u32 {
        const STATUS_ALLOWED = 0b0000_0000_0000_0001;
        const STATUS_BLOCKED = 0b0000_0000_0000_0010;

        const LOCATION_ALLOWED = 0b0000_0000_0000_0100;
        const LOCATION_BLOCKED = 0b0000_0000_0000_1000;

        const TELEGRAMS_ALLOWED = 0b0000_0000_0001_0000;
        const TELEGRAMS_BLOCKED = 0b0000_0000_0010_0000;

        const FILE_TRANSFER_ALLOWED = 0b0000_0001_0000_0000;
        const FILE_TRANSFER_BLOCKED = 0b0000_0010_0000_0000;

        const JOIN_ALLOWED = 0b0000_0000_0100_0000;
        const JOIN_BLOCKED = 0b0000_0000_1000_0000;

        const CHAT_ALLOWED = 0b0000_0100_0000_0000;
        const CHAT_BLOCKED = 0b0000_1000_0000_0000;

        const FRIEND_REQUEST_ALLOWED = 0b0001_0000_0000_0000;
        const FRIEND_REQUEST_BLOCKED = 0b0010_0000_0000_0000;

        const ALL_ALLOWED = 0b0100_0000_0000_0000;
        const ALL_BLOCKED = 0b1000_0000_0000_0000;
    }
}

impl ContactOptions {
    pub fn is_blocked(&self) -> bool {
        self.contains(ContactOptions::ALL_BLOCKED)
    }

    pub fn is_friend_request_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::FRIEND_REQUEST_ALLOWED) {
            return false;
        }

        true
    }

    pub fn is_file_transfer_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::FILE_TRANSFER_BLOCKED) {
            return false;
        }

        true
    }

    pub fn is_invite_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::JOIN_BLOCKED) {
            return false;
        }

        true
    }

    pub fn is_join_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::JOIN_BLOCKED) {
            return false;
        }

        true
    }

    pub fn is_location_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::LOCATION_BLOCKED) {
            return false;
        }

        true
    }

    pub fn is_status_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::STATUS_BLOCKED) {
            return false;
        }

        true
    }

    pub fn is_telegram_allowed(&self) -> bool {
        if self.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if self.contains(ContactOptions::TELEGRAMS_BLOCKED) {
            return false;
        }

        true
    }
}

#[derive(Debug)]
pub struct ContactQuery {
    pub citizen: u32,
    pub contact: u32,
    pub options: ContactOptions,
}

pub trait ContactDB {
    fn init_contact(&self);
    fn contact_set(&self, citizen_id: u32, contact_id: u32, options: u32)
        -> Result<(), ReasonCode>;
    fn contact_get(&self, citizen_id: u32, contact_id: u32) -> Result<ContactQuery, ReasonCode>;
    fn contact_get_all(&self, citizen_id: u32) -> Vec<ContactQuery>;
    fn contact_blocked(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_confirm_add(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_default(&self, citizen_id: u32) -> ContactQuery;
    fn contact_file_transfers_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_telegrams_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_friend_requests_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_status_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_joins_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
    fn contact_invites_allowed(&self, citizen_id: u32, contact_id: u32) -> bool;
}

impl ContactDB for Database {
    fn init_contact(&self) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_contact ( 
                Citizen int(11) unsigned NOT NULL default '0', 
                Contact int(11) unsigned NOT NULL default '0', 
                Options int(11) unsigned NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (Citizen,Contact), 
                KEY Index1 (Contact,Citizen) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }

    fn contact_set(
        &self,
        citizen_id: u32,
        contact_id: u32,
        options: u32,
    ) -> Result<(), ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        // Check if contact pair is already in the database
        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_contact WHERE Citizen=:citizen_id AND Contact=:contact_id;",
                params! {
                    "citizen_id" => citizen_id,
                    "contact_id" => contact_id,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

        if rows.is_empty() {
            // Add the contact pair if it is not already existent
            conn.exec_drop(
                r"INSERT INTO awu_contact (Citizen,Contact,Options) 
                VALUES(:citizen_id, :contact_id, :options);",
                params! {
                    "citizen_id" => citizen_id,
                    "contact_id" => contact_id,
                    "options" => options,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;
        } else {
            // Try to update the contact pair if it is already present
            conn.exec_drop(
                r"UPDATE awu_contact SET Options=:options WHERE Citizen=:citizen_id AND Contact=:contact_id;",
                params! {
                    "citizen_id" => citizen_id,
                    "contact_id" => contact_id,
                    "options" => options,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;
        }

        Ok(())
    }

    fn contact_get(&self, citizen_id: u32, contact_id: u32) -> Result<ContactQuery, ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_contact WHERE Citizen=:citizen_id AND Contact=:contact_id;",
                params! {
                    "citizen_id" => citizen_id,
                    "contact_id" => contact_id,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

        if let Some(contact) = rows.first() {
            fetch_contact(contact)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn contact_get_all(&self, citizen_id: u32) -> Vec<ContactQuery> {
        let mut result = Vec::<ContactQuery>::new();
        let mut conn = match self.conn() {
            Ok(x) => x,
            Err(_) => return result,
        };

        let rows: Vec<Row> = match conn.exec(
            r"SELECT * FROM awu_contact WHERE Citizen=:citizen_id;",
            params! {
                "citizen_id" => citizen_id,
            },
        ) {
            Ok(x) => x,
            Err(_) => return result,
        };

        for row in rows {
            if let Ok(contact) = fetch_contact(&row) {
                result.push(contact);
            }
        }

        result
    }

    fn contact_blocked(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            Err(_) => return false,
        };

        contact.options.contains(ContactOptions::ALL_BLOCKED)
    }

    fn contact_confirm_add(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            Err(_) => return false,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return true;
        }

        if contact
            .options
            .contains(ContactOptions::FRIEND_REQUEST_BLOCKED)
        {
            return true;
        }

        false
    }

    fn contact_default(&self, citizen_id: u32) -> ContactQuery {
        match self.contact_get(citizen_id, 0) {
            Ok(contact) => contact,
            Err(_) => ContactQuery {
                citizen: citizen_id,
                contact: 0,
                options: ContactOptions::default(),
            },
        }
    }

    fn contact_file_transfers_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = self
            .contact_get(citizen_id, contact_id)
            .unwrap_or_else(|_| self.contact_default(citizen_id));

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if contact
            .options
            .contains(ContactOptions::FILE_TRANSFER_BLOCKED)
        {
            return false;
        }

        true
    }

    fn contact_telegrams_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = self
            .contact_get(citizen_id, contact_id)
            .unwrap_or_else(|_| self.contact_default(citizen_id));

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if contact.options.contains(ContactOptions::TELEGRAMS_BLOCKED) {
            return false;
        }

        true
    }

    fn contact_friend_requests_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            _ => return true,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if contact
            .options
            .contains(ContactOptions::FRIEND_REQUEST_BLOCKED)
        {
            return false;
        }

        true
    }

    fn contact_status_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            _ => return true,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return false;
        }

        if contact.options.contains(ContactOptions::STATUS_BLOCKED) {
            return false;
        }

        true
    }

    fn contact_joins_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            _ => return true,
        };

        contact.options.is_join_allowed()
    }

    fn contact_invites_allowed(&self, citizen_id: u32, contact_id: u32) -> bool {
        let contact = match self.contact_get(citizen_id, contact_id) {
            Ok(x) => x,
            _ => return true,
        };

        contact.options.is_invite_allowed()
    }
}

fn fetch_contact(row: &Row) -> Result<ContactQuery, ReasonCode> {
    let citizen: u32 = database::fetch_int(row, "Citizen")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let contact: u32 = database::fetch_int(row, "Contact")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let options: u32 = database::fetch_int(row, "Options")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    Ok(ContactQuery {
        citizen,
        contact,
        options: ContactOptions::from_bits_truncate(options),
    })
}
