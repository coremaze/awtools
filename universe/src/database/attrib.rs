use std::collections::HashMap;

use crate::config::UniverseConfig;

use super::{fetch_int, fetch_string, Database};
use aw_core::ReasonCode;
use mysql::prelude::*;
use mysql::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

type Result<T, E> = std::result::Result<T, E>;

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
    UnknownUniverseSetting = 23,
}

pub trait AttribDB {
    fn init_attrib(&self, universe_config: &UniverseConfig);
    fn attrib_set(&self, attribute_id: Attribute, value: &str) -> Result<(), ReasonCode>;
    fn attrib_get(&self) -> Result<HashMap<Attribute, String>, ReasonCode>;
}

impl AttribDB for Database {
    fn init_attrib(&self, universe_config: &UniverseConfig) {
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_attrib ( 
            ID int(11) NOT NULL default '0', 
            Changed tinyint(1) NOT NULL default '0', 
            Value varchar(255) NOT NULL default '', 
            PRIMARY KEY  (ID) 
        ) ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();

        // Unimplemented: mail template
        // Unimplemented: mail file
        // Unimplemented: mail command

        self.attrib_set(Attribute::Userlist, bool_attrib(universe_config.user_list))
            .expect("Failed to set userlist attribute.");

        self.attrib_set(
            Attribute::CitizenChanges,
            bool_attrib(universe_config.allow_citizen_changes),
        )
        .expect("Failed to set citizenchanges attribute.");
    }

    fn attrib_set(&self, attribute_id: Attribute, value: &str) -> Result<(), ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        // Check if attribute is already in the database
        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_attrib WHERE ID=:id",
                params! {
                    "id" => attribute_id as u32,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

        if rows.len() == 0 {
            // Add the attribute if it is not already existent
            conn.exec_drop(
                r"INSERT INTO awu_attrib (ID, Value) VALUES(:id, :value);",
                params! {
                    "value" => value,
                    "id" => attribute_id as u32,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;
            log::debug!("Set attribute {attribute_id:?} to {value}");
        } else {
            // Try to update the attribute if it is already present
            conn.exec_drop(
                r"UPDATE awu_attrib SET Value=:value, Changed=NOT Changed WHERE ID=:id;",
                params! {
                    "value" => value,
                    "id" => attribute_id as u32,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;
            log::debug!("Updated attribute {attribute_id:?} to {value}");
        }

        Ok(())
    }

    fn attrib_get(&self) -> Result<HashMap<Attribute, String>, ReasonCode> {
        let mut result = HashMap::<Attribute, String>::new();
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        // Get all attributes from database
        let rows: Vec<Row> = conn
            .exec(r"SELECT * FROM awu_attrib;", Params::Empty)
            .map_err(|_| ReasonCode::DatabaseError)?;

        // Add each valid response to the result
        for row in &rows {
            let id = fetch_int(row, "ID").ok_or(ReasonCode::DatabaseError)?;

            let value = fetch_string(row, "Value").ok_or(ReasonCode::DatabaseError)?;

            // Convert numeric ID back to Attributes
            if let Some(attribute) = Attribute::from_i64(id) {
                result.insert(attribute, value);
            }
        }

        Ok(result)
    }
}

fn bool_attrib(value: bool) -> &'static str {
    match value {
        true => "Y",
        false => "N",
    }
}
