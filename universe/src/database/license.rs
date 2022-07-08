use std::time::{SystemTime, UNIX_EPOCH};

use crate::database;

use super::Database;
use aw_core::ReasonCode;
use mysql::prelude::*;
use mysql::*;

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
        let mut conn = self
            .pool
            .get_conn()
            .expect("Could not get mysql connection.");

        // "Range" has been changed to "WorldSize" because range is now a keyword.
        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_license ( 
                ID int(11) NOT NULL auto_increment, 
                Name varchar(50) NOT NULL default '', 
                Password varchar(255) NOT NULL default '', 
                Email varchar(255) NOT NULL default '', 
                Comment varchar(255) NOT NULL default '', 
                Creation int(11) NOT NULL default '0', 
                Expiration int(11) NOT NULL default '0', 
                LastStart int(11) NOT NULL default '0', 
                LastAddress int(11) NOT NULL default '0', 
                Users int(11) NOT NULL default '0', 
                WorldSize int(11) NOT NULL default '0', 
                Hidden tinyint(1) NOT NULL default '0', 
                Changed tinyint(1) NOT NULL default '0', 
                Tourists tinyint(1) NOT NULL default '0', 
                Voip tinyint(1) NOT NULL default '0', 
                Plugins tinyint(1) NOT NULL default '0', 
                PRIMARY KEY  (ID), UNIQUE KEY Index1 (Name) 
            ) 
            ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();
    }

    fn license_by_name(&self, name: &str) -> Result<LicenseQuery, ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_license WHERE Name=:name",
                params! {
                    "name" => name,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

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
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        conn.exec_drop(
            r"INSERT INTO awu_license(Creation, Expiration, LastStart, LastAddress, Hidden,
                Tourists, Users, WorldSize, Voip, Plugins, Name, Password, Email, Comment) 
                VALUES(:creation, :expiration, :last_start, :last_address, :hidden, :tourists,
                    :users, :world_size, :voip, :plugins, :name, :password, :email, :comment);",
            params! {
                "creation" => now,
                "expiration" => lic.expiration,
                "last_start" => 0,
                "last_address" => 0,
                "hidden" => lic.hidden,
                "tourists" => lic.tourists,
                "users" => lic.users,
                "world_size" => lic.world_size,
                "voip" => lic.voip,
                "plugins" => lic.plugins,
                "name" => &lic.name,
                "password" => &lic.password,
                "email" => &lic.email,
                "comment" => &lic.comment,
            },
        )
        .map_err(|_| ReasonCode::DatabaseError)?;

        Ok(())
    }

    fn license_next(&self, name: &str) -> Result<LicenseQuery, ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_license WHERE Name>:name ORDER BY Name LIMIT 1",
                params! {
                    "name" => name,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

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
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_license WHERE Name<:name ORDER BY Name DESC LIMIT 1",
                params! {
                    "name" => name,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

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
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        conn.exec_drop(
            r"UPDATE awu_license
            SET Changed=NOT Changed, Creation=:creation, Expiration=:expiration, LastStart=:last_start, 
            LastAddress=:last_address, Hidden=:hidden, Tourists=:tourists, Users=:users,
            WorldSize=:world_size, Voip=:voip, Plugins=:plugins, Password=:password, 
            Email=:email, Comment=:comment 
            WHERE Name=:name;",
            params! {
                "creation" => lic.creation,
                "expiration" => lic.expiration,
                "last_start" => lic.last_start,
                "last_address" => lic.last_address,
                "hidden" => lic.hidden,
                "tourists" => lic.tourists,
                "users" => lic.users,
                "world_size" => lic.world_size,
                "voip" => lic.voip,
                "plugins" => lic.plugins,
                "name" => &lic.name,
                "password" => &lic.password,
                "email" => &lic.email,
                "comment" => &lic.comment,
            },
        )
        .map_err(|_| ReasonCode::DatabaseError)?;

        Ok(())
    }
}

fn fetch_license(row: &Row) -> Result<LicenseQuery, ReasonCode> {
    let id: u32 = database::fetch_int(row, "ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let name: String = database::fetch_string(row, "Name").ok_or(ReasonCode::DatabaseError)?;

    let password: String =
        database::fetch_string(row, "Password").ok_or(ReasonCode::DatabaseError)?;

    let email: String = database::fetch_string(row, "Email").ok_or(ReasonCode::DatabaseError)?;

    let comment: String =
        database::fetch_string(row, "Comment").ok_or(ReasonCode::DatabaseError)?;

    let creation: u32 = database::fetch_int(row, "Creation")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let expiration: u32 = database::fetch_int(row, "Expiration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_start: u32 = database::fetch_int(row, "LastStart")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_address: u32 = database::fetch_int(row, "LastAddress")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let users: u32 = database::fetch_int(row, "Users")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let world_size: u32 = database::fetch_int(row, "WorldSize")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let hidden: u32 = database::fetch_int(row, "Hidden")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let changed: u32 = database::fetch_int(row, "Changed")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let tourists: u32 = database::fetch_int(row, "Tourists")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let voip: u32 = database::fetch_int(row, "Voip")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let plugins: u32 = database::fetch_int(row, "Plugins")
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
