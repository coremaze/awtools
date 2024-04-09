use std::time::{SystemTime, UNIX_EPOCH};

use crate::aw_params;

use super::{AWRow, Database};
use aw_core::ReasonCode;

type Result<T, E> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct LicenseQuery {
    pub id: u32,
    pub name: String,
    pub password: String,
    pub email: String,
    pub comment: String,
    pub creation: u32,
    pub expiration: u32,
    pub last_start: u32,
    pub last_address: u32,
    pub users: u32,
    pub world_size: u32,
    pub hidden: u32,
    pub changed: u32,
    pub tourists: u32,
    pub voip: u32,
    pub plugins: u32,
}

pub trait LicenseDB {
    fn init_license(&self);
    fn license_by_name(&self, name: &str) -> Result<LicenseQuery, ReasonCode>;
    fn license_add(&self, lic: &LicenseQuery) -> Result<(), ReasonCode>;
    fn license_next(&self, name: &str) -> Result<LicenseQuery, ReasonCode>;
    fn license_prev(&self, name: &str) -> Result<LicenseQuery, ReasonCode>;
    fn license_change(&self, lic: &LicenseQuery) -> Result<(), ReasonCode>;
}

impl LicenseDB for Database {
    fn init_license(&self) {
        let auto_increment_not_null = self.auto_increment_not_null();
        // "Range" has been changed to "WorldSize" because range is now a keyword.
        self.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_license ( 
                ID INTEGER PRIMARY KEY {auto_increment_not_null}, 
                Name varchar(50) NOT NULL default '', 
                Password varchar(255) NOT NULL default '', 
                Email varchar(255) NOT NULL default '', 
                Comment varchar(255) NOT NULL default '', 
                Creation INTEGER NOT NULL default '0', 
                Expiration INTEGER NOT NULL default '0', 
                LastStart INTEGER NOT NULL default '0', 
                LastAddress INTEGER NOT NULL default '0', 
                Users INTEGER NOT NULL default '0', 
                WorldSize INTEGER NOT NULL default '0', 
                Hidden tinyint(1) NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0', 
                Tourists tinyint(1) NOT NULL default '0', 
                Voip tinyint(1) NOT NULL default '0', 
                Plugins tinyint(1) NOT NULL default '0'
            );"
            ),
            vec![],
        );
    }

    fn license_by_name(&self, name: &str) -> Result<LicenseQuery, ReasonCode> {
        let rows = self.exec(
            r"SELECT * FROM awu_license WHERE Name=?",
            aw_params! {
                name
            },
        );

        if rows.len() > 1 {
            return Err(ReasonCode::DatabaseError);
        }

        if let Some(lic) = rows.first() {
            fetch_license(lic)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn license_add(&self, lic: &LicenseQuery) -> Result<(), ReasonCode> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        self.exec(
            r"INSERT INTO awu_license(Creation, Expiration, LastStart, LastAddress, Hidden,
                Tourists, Users, WorldSize, Voip, Plugins, Name, Password, Email, Comment) 
                VALUES(?, ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?, ?, ?, ?);",
            aw_params! {
                now,
                lic.expiration,
                0,
                0,
                lic.hidden,
                lic.tourists,
                lic.users,
                lic.world_size,
                lic.voip,
                lic.plugins,
                &lic.name,
                &lic.password,
                &lic.email,
                &lic.comment
            },
        );

        Ok(())
    }

    fn license_next(&self, name: &str) -> Result<LicenseQuery, ReasonCode> {
        let rows = self.exec(
            r"SELECT * FROM awu_license WHERE Name>? ORDER BY Name LIMIT 1",
            aw_params! {
                name
            },
        );

        if rows.len() > 1 {
            return Err(ReasonCode::DatabaseError);
        }

        if let Some(lic) = rows.first() {
            fetch_license(lic)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn license_prev(&self, name: &str) -> Result<LicenseQuery, ReasonCode> {
        let rows = self.exec(
            r"SELECT * FROM awu_license WHERE Name<? ORDER BY Name DESC LIMIT 1",
            aw_params! {
                name
            },
        );

        if rows.len() > 1 {
            return Err(ReasonCode::DatabaseError);
        }

        if let Some(lic) = rows.first() {
            fetch_license(lic)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn license_change(&self, lic: &LicenseQuery) -> Result<(), ReasonCode> {
        self.exec(
            r"UPDATE awu_license
            SET Changed=NOT Changed, Creation=?, Expiration=?, LastStart=?, 
            LastAddress=?, Hidden=?, Tourists=?, Users=?,
            WorldSize=?, Voip=?, Plugins=?, Password=?, 
            Email=?, Comment=? 
            WHERE Name=?;",
            aw_params! {
                lic.creation,
                lic.expiration,
                lic.last_start,
                lic.last_address,
                lic.hidden,
                lic.tourists,
                lic.users,
                lic.world_size,
                lic.voip,
                lic.plugins,
                &lic.password,
                &lic.email,
                &lic.comment,
                &lic.name
            },
        );

        Ok(())
    }
}

fn fetch_license(row: &AWRow) -> Result<LicenseQuery, ReasonCode> {
    let id: u32 = row
        .fetch_int("ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let name: String = row.fetch_string("Name").ok_or(ReasonCode::DatabaseError)?;

    let password: String = row
        .fetch_string("Password")
        .ok_or(ReasonCode::DatabaseError)?;

    let email: String = row.fetch_string("Email").ok_or(ReasonCode::DatabaseError)?;

    let comment: String = row
        .fetch_string("Comment")
        .ok_or(ReasonCode::DatabaseError)?;

    let creation: u32 = row
        .fetch_int("Creation")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let expiration: u32 = row
        .fetch_int("Expiration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_start: u32 = row
        .fetch_int("LastStart")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    // It is not unexpected for LastAddress to end up in the database as a negative number.
    let last_address: u32 = row
        .fetch_int("LastAddress")
        .ok_or(ReasonCode::DatabaseError)? as u32;

    let users: u32 = row
        .fetch_int("Users")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let world_size: u32 = row
        .fetch_int("WorldSize")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let hidden: u32 = row
        .fetch_int("Hidden")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let changed: u32 = row
        .fetch_int("Changed")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let tourists: u32 = row
        .fetch_int("Tourists")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let voip: u32 = row
        .fetch_int("Voip")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let plugins: u32 = row
        .fetch_int("Plugins")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    Ok(LicenseQuery {
        id,
        name,
        password,
        email,
        comment,
        expiration,
        last_start,
        last_address,
        users,
        world_size,
        hidden,
        changed,
        tourists,
        voip,
        plugins,
        creation,
    })
}
