use std::collections::HashMap;

use crate::configuration::UniverseConfig;

use aw_db::{aw_params, DatabaseResult};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::UniverseDatabase;

#[derive(Clone, Copy, Debug, FromPrimitive, Eq, Hash, PartialEq)]
pub enum Attribute {
    AllowTourists = 0,
    UnknownBilling1 = 1,
    BetaBrowser = 2,
    MinimumBrowser = 3,
    LatestBrowser = 4,
    UniverseBuild = 5,
    CitizenChanges = 6,
    UnknownBilling7 = 7,
    BillingMethod = 8,
    BillingUnknown9 = 9,
    SearchTabURL = 10,
    Timestamp = 11,
    WelcomeMessage = 12,
    BetaWorld = 13,
    MinimumWorld = 14,
    LatestWorld = 15,
    DefaultStartWorld = 16,
    Userlist = 17,
    NotepadTabURL = 18,
    MailTemplate = 19,
    MailFile = 20,
    MailCommand = 21,
    PAVObjectPath = 22,
    TextureAndSeqObjectPath = 23,
    ObjectRefresh = 24,
    PAVObjectPasswordObfuscated = 25,
}

pub trait AttribDB {
    fn init_attrib(&self, universe_config: &UniverseConfig) -> DatabaseResult<()>;
    fn attrib_set(&self, attribute_id: Attribute, value: &str) -> DatabaseResult<()>;
    fn attrib_get(&self) -> DatabaseResult<HashMap<Attribute, String>>;
}

impl AttribDB for UniverseDatabase {
    fn init_attrib(&self, universe_config: &UniverseConfig) -> DatabaseResult<()> {
        let r = self.db.exec(
            r"CREATE TABLE IF NOT EXISTS awu_attrib ( 
            ID INTEGER PRIMARY KEY NOT NULL default '0', 
            Changed tinyint(1) NOT NULL default '0', 
            Value varchar(255) NOT NULL default ''
        );",
            vec![],
        );

        if r.is_err() {
            return DatabaseResult::DatabaseError;
        }

        // Unimplemented: mail template
        // Unimplemented: mail file
        // Unimplemented: mail command

        if self
            .attrib_set(Attribute::Userlist, bool_attrib(universe_config.user_list))
            .is_err()
        {
            return DatabaseResult::DatabaseError;
        }

        if self
            .attrib_set(
                Attribute::CitizenChanges,
                bool_attrib(universe_config.allow_citizen_changes),
            )
            .is_err()
        {
            return DatabaseResult::DatabaseError;
        }

        DatabaseResult::Ok(())
    }

    fn attrib_set(&self, attribute_id: Attribute, value: &str) -> DatabaseResult<()> {
        // Check if attribute is already in the database
        let rows = match self.db.exec(
            r"SELECT * FROM awu_attrib WHERE ID=?",
            aw_params!(attribute_id as u32),
        ) {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.is_empty() {
            // Add the attribute if it is not already existent
            let r = self.db.exec(
                r"INSERT INTO awu_attrib (ID, Value) VALUES(?, ?);",
                aw_params!(attribute_id as u32, value),
            );

            match r {
                DatabaseResult::Ok(_) => {}
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            }
            log::debug!("Set attribute {attribute_id:?} to {value}");
        } else {
            // Try to update the attribute if it is already present
            let r = self.db.exec(
                r"UPDATE awu_attrib SET Value=?, Changed=NOT Changed WHERE ID=?;",
                aw_params! {
                    value,
                    attribute_id as u32
                },
            );

            match r {
                DatabaseResult::Ok(_) => {}
                DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
            }

            log::debug!("Updated attribute {attribute_id:?} to {value}");
        }

        DatabaseResult::Ok(())
    }

    fn attrib_get(&self) -> DatabaseResult<HashMap<Attribute, String>> {
        let mut result = HashMap::<Attribute, String>::new();

        // Get all attributes from database
        log::trace!("getting rows");

        let rows = match self.db.exec(r"SELECT * FROM awu_attrib;", vec![]) {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };
        log::trace!("rows {rows:?}");

        // Add each valid response to the result
        for row in &rows {
            log::trace!("get id {:?}", row.fetch_int("ID"));

            let id = match row.fetch_int("ID") {
                Some(value) => value,
                None => return DatabaseResult::DatabaseError,
            };

            log::trace!("get value {:?}", row.fetch_string("Value"));

            let value = match row.fetch_string("Value") {
                Some(value) => value,
                None => return DatabaseResult::DatabaseError,
            };

            // Convert numeric ID back to Attributes
            if let Some(attribute) = Attribute::from_i64(id) {
                result.insert(attribute, value);
            }
        }

        DatabaseResult::Ok(result)
    }
}

fn bool_attrib(value: bool) -> &'static str {
    match value {
        true => "Y",
        false => "N",
    }
}
