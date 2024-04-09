use std::collections::HashMap;

use crate::aw_params;
use crate::configuration::UniverseConfig;

use super::Database;
use aw_core::ReasonCode;

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
    TextureAndSeqObjectPath = 23,
}

pub trait AttribDB {
    fn init_attrib(&self, universe_config: &UniverseConfig);
    fn attrib_set(&self, attribute_id: Attribute, value: &str) -> Result<(), ReasonCode>;
    fn attrib_get(&self) -> Result<HashMap<Attribute, String>, ReasonCode>;
}

impl AttribDB for Database {
    fn init_attrib(&self, universe_config: &UniverseConfig) {
        self.exec(
            r"CREATE TABLE IF NOT EXISTS awu_attrib ( 
            ID INTEGER PRIMARY KEY NOT NULL default '0', 
            Changed tinyint(1) NOT NULL default '0', 
            Value varchar(255) NOT NULL default ''
        );",
            vec![],
        );

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
        // Check if attribute is already in the database
        let rows = self.exec(
            r"SELECT * FROM awu_attrib WHERE ID=?",
            aw_params!(attribute_id as u32),
        );

        if rows.is_empty() {
            // Add the attribute if it is not already existent
            self.exec(
                r"INSERT INTO awu_attrib (ID, Value) VALUES(?, ?);",
                aw_params!(attribute_id as u32, value),
            );
            log::debug!("Set attribute {attribute_id:?} to {value}");
        } else {
            // Try to update the attribute if it is already present
            self.exec(
                r"UPDATE awu_attrib SET Value=?, Changed=NOT Changed WHERE ID=?;",
                aw_params! {
                    value,
                    attribute_id as u32
                },
            );
            log::debug!("Updated attribute {attribute_id:?} to {value}");
        }

        Ok(())
    }

    fn attrib_get(&self) -> Result<HashMap<Attribute, String>, ReasonCode> {
        let mut result = HashMap::<Attribute, String>::new();

        // Get all attributes from database
        log::trace!("getting rows");

        let rows = self.exec(r"SELECT * FROM awu_attrib;", vec![]);
        log::trace!("rows {rows:?}");

        // Add each valid response to the result
        for row in &rows {
            log::trace!("get id {:?}", row.fetch_int("ID"));
            let id = row.fetch_int("ID").ok_or(ReasonCode::DatabaseError)?;

            log::trace!("get value {:?}", row.fetch_string("Value"));

            let value = row.fetch_string("Value").ok_or(ReasonCode::DatabaseError)?;

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
