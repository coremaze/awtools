use aw_db::{aw_params, DatabaseResult, Row};
use bitflags::bitflags;

use super::UniverseDatabase;

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
    /// Returns a new ContactOptions, keeping the old options where the applied options are neither allowed nor blocked
    pub fn apply_changes(&self, other: ContactOptions) -> ContactOptions {
        let mut new_options = *self;

        if other.contains(ContactOptions::STATUS_ALLOWED)
            || other.contains(ContactOptions::STATUS_BLOCKED)
        {
            new_options.set(
                ContactOptions::STATUS_ALLOWED,
                other.contains(ContactOptions::STATUS_ALLOWED),
            );

            new_options.set(
                ContactOptions::STATUS_BLOCKED,
                other.contains(ContactOptions::STATUS_BLOCKED),
            );
        }

        if other.contains(ContactOptions::LOCATION_ALLOWED)
            || other.contains(ContactOptions::LOCATION_BLOCKED)
        {
            new_options.set(
                ContactOptions::LOCATION_ALLOWED,
                other.contains(ContactOptions::LOCATION_ALLOWED),
            );

            new_options.set(
                ContactOptions::LOCATION_BLOCKED,
                other.contains(ContactOptions::LOCATION_BLOCKED),
            );
        }

        if other.contains(ContactOptions::TELEGRAMS_ALLOWED)
            || other.contains(ContactOptions::TELEGRAMS_BLOCKED)
        {
            new_options.set(
                ContactOptions::TELEGRAMS_ALLOWED,
                other.contains(ContactOptions::TELEGRAMS_ALLOWED),
            );

            new_options.set(
                ContactOptions::TELEGRAMS_BLOCKED,
                other.contains(ContactOptions::TELEGRAMS_BLOCKED),
            );
        }

        if other.contains(ContactOptions::FILE_TRANSFER_ALLOWED)
            || other.contains(ContactOptions::FILE_TRANSFER_BLOCKED)
        {
            new_options.set(
                ContactOptions::FILE_TRANSFER_ALLOWED,
                other.contains(ContactOptions::FILE_TRANSFER_ALLOWED),
            );

            new_options.set(
                ContactOptions::FILE_TRANSFER_BLOCKED,
                other.contains(ContactOptions::FILE_TRANSFER_BLOCKED),
            );
        }

        if other.contains(ContactOptions::JOIN_ALLOWED)
            || other.contains(ContactOptions::JOIN_BLOCKED)
        {
            new_options.set(
                ContactOptions::JOIN_ALLOWED,
                other.contains(ContactOptions::JOIN_ALLOWED),
            );

            new_options.set(
                ContactOptions::JOIN_BLOCKED,
                other.contains(ContactOptions::JOIN_BLOCKED),
            );
        }

        if other.contains(ContactOptions::CHAT_ALLOWED)
            || other.contains(ContactOptions::CHAT_BLOCKED)
        {
            new_options.set(
                ContactOptions::CHAT_ALLOWED,
                other.contains(ContactOptions::CHAT_ALLOWED),
            );

            new_options.set(
                ContactOptions::CHAT_BLOCKED,
                other.contains(ContactOptions::CHAT_BLOCKED),
            );
        }

        if other.contains(ContactOptions::FRIEND_REQUEST_ALLOWED)
            || other.contains(ContactOptions::FRIEND_REQUEST_BLOCKED)
        {
            new_options.set(
                ContactOptions::FRIEND_REQUEST_ALLOWED,
                other.contains(ContactOptions::FRIEND_REQUEST_ALLOWED),
            );

            new_options.set(
                ContactOptions::FRIEND_REQUEST_BLOCKED,
                other.contains(ContactOptions::FRIEND_REQUEST_BLOCKED),
            );
        }

        new_options
    }

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
    fn init_contact(&self) -> DatabaseResult<()>;
    fn contact_set(&self, citizen_id: u32, contact_id: u32, options: u32) -> DatabaseResult<()>;
    fn contact_get(&self, citizen_id: u32, contact_id: u32)
        -> DatabaseResult<Option<ContactQuery>>;
    fn contact_get_all(&self, citizen_id: u32) -> DatabaseResult<Vec<ContactQuery>>;
    fn contact_blocked(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_confirm_add(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_default(&self, citizen_id: u32) -> DatabaseResult<ContactQuery>;
    fn contact_file_transfers_allowed(
        &self,
        citizen_id: u32,
        contact_id: u32,
    ) -> DatabaseResult<bool>;
    fn contact_telegrams_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_friend_requests_allowed(
        &self,
        citizen_id: u32,
        contact_id: u32,
    ) -> DatabaseResult<bool>;
    fn contact_status_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_joins_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_invites_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool>;
    fn contact_delete(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<()>;
}

impl ContactDB for UniverseDatabase {
    fn init_contact(&self) -> DatabaseResult<()> {
        let unsigned = self.db.unsigned_str();
        let r = self.db.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_contact ( 
                Citizen INTEGER {unsigned} NOT NULL default '0', 
                Contact INTEGER {unsigned} NOT NULL default '0', 
                Options INTEGER {unsigned} NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0',
                PRIMARY KEY (Citizen, Contact)
            );"
            ),
            vec![],
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn contact_set(&self, citizen_id: u32, contact_id: u32, options: u32) -> DatabaseResult<()> {
        // Check if contact pair is already in the database
        let r = self.db.exec(
            r"SELECT * FROM awu_contact WHERE Citizen=? AND Contact=?;",
            aw_params! {
                citizen_id,
                contact_id
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let r = if rows.is_empty() {
            // Add the contact pair if it is not already existent
            self.db.exec(
                r"INSERT INTO awu_contact (Citizen,Contact,Options) 
                VALUES(?, ?, ?);",
                aw_params! {
                    citizen_id,
                    contact_id,
                    options
                },
            )
        } else {
            // Try to update the contact pair if it is already present
            self.db.exec(
                r"UPDATE awu_contact SET Options=? WHERE Citizen=? AND Contact=?;",
                aw_params! {
                    options,
                    citizen_id,
                    contact_id
                },
            )
        };

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn contact_get(
        &self,
        citizen_id: u32,
        contact_id: u32,
    ) -> DatabaseResult<Option<ContactQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_contact WHERE Citizen=? AND Contact=?;",
            aw_params! {
                citizen_id,
                contact_id
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let Some(contact) = rows.first() else {
            // No such contact
            return DatabaseResult::Ok(None);
        };

        match fetch_contact(contact) {
            DatabaseResult::Ok(contact) => DatabaseResult::Ok(Some(contact)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn contact_get_all(&self, citizen_id: u32) -> DatabaseResult<Vec<ContactQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_contact WHERE Citizen=?;",
            aw_params! {
                citizen_id
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let mut result = Vec::<ContactQuery>::new();
        for row in rows {
            match fetch_contact(&row) {
                DatabaseResult::Ok(contact) => result.push(contact),
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            }
        }

        DatabaseResult::Ok(result)
    }

    fn contact_blocked(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => {
                DatabaseResult::Ok(contact.options.contains(ContactOptions::ALL_BLOCKED))
            }
            DatabaseResult::Ok(None) => DatabaseResult::Ok(false),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn contact_confirm_add(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(contact) => contact,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let Some(contact) = contact else {
            return DatabaseResult::Ok(false);
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return DatabaseResult::Ok(true);
        }

        if contact
            .options
            .contains(ContactOptions::FRIEND_REQUEST_BLOCKED)
        {
            return DatabaseResult::Ok(true);
        }

        DatabaseResult::Ok(false)
    }

    fn contact_default(&self, citizen_id: u32) -> DatabaseResult<ContactQuery> {
        let contact = match self.contact_get(citizen_id, 0) {
            DatabaseResult::Ok(contact) => contact,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        let default_contact = match contact {
            Some(contact) => contact,
            None => ContactQuery {
                citizen: citizen_id,
                contact: 0,
                options: ContactOptions::default(),
            },
        };

        DatabaseResult::Ok(default_contact)
    }

    fn contact_file_transfers_allowed(
        &self,
        citizen_id: u32,
        contact_id: u32,
    ) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        if contact
            .options
            .contains(ContactOptions::FILE_TRANSFER_BLOCKED)
        {
            return DatabaseResult::Ok(false);
        }

        DatabaseResult::Ok(true)
    }

    fn contact_telegrams_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        if contact.options.contains(ContactOptions::TELEGRAMS_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        DatabaseResult::Ok(true)
    }

    fn contact_friend_requests_allowed(
        &self,
        citizen_id: u32,
        contact_id: u32,
    ) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        if contact
            .options
            .contains(ContactOptions::FRIEND_REQUEST_BLOCKED)
        {
            return DatabaseResult::Ok(false);
        }

        DatabaseResult::Ok(true)
    }

    fn contact_status_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if contact.options.contains(ContactOptions::ALL_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        if contact.options.contains(ContactOptions::STATUS_BLOCKED) {
            return DatabaseResult::Ok(false);
        }

        DatabaseResult::Ok(true)
    }

    fn contact_joins_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        DatabaseResult::Ok(contact.options.is_join_allowed())
    }

    fn contact_invites_allowed(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<bool> {
        let contact = match self.contact_get(citizen_id, contact_id) {
            DatabaseResult::Ok(Some(contact)) => contact,
            DatabaseResult::Ok(None) => match self.contact_default(citizen_id) {
                DatabaseResult::Ok(default_contact) => default_contact,
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            },
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        DatabaseResult::Ok(contact.options.is_invite_allowed())
    }

    fn contact_delete(&self, citizen_id: u32, contact_id: u32) -> DatabaseResult<()> {
        let r = self.db.exec(
            r"DELETE FROM awu_contact WHERE Citizen=?  AND Contact=?;",
            aw_params! {
                citizen_id,
                contact_id
            },
        );

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}

fn fetch_contact(row: &Row) -> DatabaseResult<ContactQuery> {
    let citizen = match row.fetch_int("Citizen").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let contact = match row.fetch_int("Contact").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let options = match row.fetch_int("Options").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    DatabaseResult::Ok(ContactQuery {
        citizen,
        contact,
        options: ContactOptions::from_bits_truncate(options),
    })
}
