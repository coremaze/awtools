use crate::database;

use super::Database;
use aw_core::ReasonCode;
use mysql::*;
use mysql::{params, prelude::*};
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T, E> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct CitizenQuery {
    pub id: u32,
    pub changed: u32,
    pub name: String,
    pub password: String,
    pub email: String,
    pub priv_pass: String,
    pub comment: String,
    pub url: String,
    pub immigration: u32,
    pub expiration: u32,
    pub last_login: u32,
    pub last_address: u32,
    pub total_time: u32,
    pub bot_limit: u32,
    pub beta: u32,
    pub cav_enabled: u32,
    pub cav_template: u32,
    pub enabled: u32,
    pub privacy: u32,
    pub trial: u32,
}

pub trait CitizenDB {
    fn init_citizen(&self);
    fn citizen_by_number(&self, citizen_id: u32) -> Result<CitizenQuery, ReasonCode>;
    fn citizen_add(&self, citizen: &CitizenQuery) -> Result<(), ReasonCode>;
}

impl CitizenDB for Database {
    fn init_citizen(&self) {
        let mut conn = self.conn().expect("Could not get mysql connection.");

        conn.query_drop(
            r"CREATE TABLE IF NOT EXISTS awu_citizen ( 
            ID int(11) unsigned NOT NULL auto_increment, 
            Changed tinyint(1) NOT NULL default '0', 
            Name varchar(255) NOT NULL default '', 
            Password varchar(255) NOT NULL default '', 
            Email varchar(255) NOT NULL default '', 
            PrivPass varchar(255) NOT NULL default '', 
            Comment varchar(255) NOT NULL default '', 
            URL varchar(255) NOT NULL default '', 
            Immigration int(11) NOT NULL default '0', 
            Expiration int(11) NOT NULL default '0', 
            LastLogin int(11) NOT NULL default '0', 
            LastAddress int(11) NOT NULL default '0', 
            TotalTime int(11) NOT NULL default '0', 
            BotLimit int(11) NOT NULL default '0', 
            Beta tinyint(1) NOT NULL default '0', 
            CAVEnabled tinyint(1) NOT NULL default '0', 
            CAVTemplate int(11) NOT NULL default '0', 
            Enabled tinyint(1) NOT NULL default '1', 
            Privacy int(11) NOT NULL default '0', 
            Trial tinyint(1) NOT NULL default '0', 
            PRIMARY KEY  (ID), 
            UNIQUE KEY Index1 (Name), 
            KEY Index2 (Email) 
        ) 
        ENGINE=MyISAM DEFAULT CHARSET=latin1;",
        )
        .unwrap();

        // Create default Administrator account if one doesn't exist yet
        if self.citizen_by_number(1).is_err() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current time is before the unix epoch.")
                .as_secs();

            let admin = CitizenQuery {
                id: 1,
                changed: 0,
                name: "Administrator".to_string(),
                password: "welcome".to_string(),
                //email: "support@activeworlds.com".to_string(),
                email: Default::default(),
                priv_pass: Default::default(),
                comment: Default::default(),
                url: Default::default(),
                immigration: now as u32,
                expiration: 0,
                last_login: 0,
                last_address: 0,
                total_time: 0,
                bot_limit: 3,
                beta: 0,
                cav_enabled: 0,
                cav_template: 0,
                enabled: 1,
                privacy: 0,
                trial: 0,
            };

            match self.citizen_add(&admin) {
                Ok(_) => println!("Citizen #1 created as {} / {}", admin.name, admin.password),
                Err(_) => eprintln!("Failed to create citizen #1"),
            };
        }
    }

    fn citizen_by_number(&self, citizen_id: u32) -> Result<CitizenQuery, ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        let rows: Vec<Row> = conn
            .exec(
                r"SELECT * FROM awu_citizen WHERE ID=:id",
                params! {
                    "id" => citizen_id,
                },
            )
            .map_err(|_| ReasonCode::DatabaseError)?;

        if rows.len() > 1 {
            return Err(ReasonCode::DatabaseError);
        }

        if let Some(user) = rows.first() {
            fetch_citizen(user)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn citizen_add(&self, citizen: &CitizenQuery) -> Result<(), ReasonCode> {
        let mut conn = self.conn().map_err(|_| ReasonCode::DatabaseError)?;

        conn.exec_drop(
            r"INSERT INTO awu_citizen(
                ID, Immigration, Expiration, LastLogin, LastAddress, TotalTime, 
                BotLimit, Beta, Enabled, Trial, Privacy, CAVEnabled, CAVTemplate, 
                Name, Password, Email, PrivPass, Comment, URL) 
            VALUES(:id, :immigration, :expiration, :last_login, :last_address, :total_time, 
                :bot_limit, :beta, :enabled, :trial, :privacy, :cav_enabled, :cav_template, 
                :name, :password, :email, :priv_pass, :comment, :url)",
            params! {
                "id" => citizen.id,
                "immigration" => citizen.immigration,
                "expiration" => citizen.expiration,
                "last_login" => citizen.last_login,
                "last_address" => citizen.last_address,
                "total_time" => citizen.total_time,
                "bot_limit" => citizen.bot_limit,
                "beta" => citizen.beta,
                "enabled" => citizen.enabled,
                "trial" => citizen.trial,
                "privacy" => citizen.privacy,
                "cav_enabled" => citizen.cav_enabled,
                "cav_template" => citizen.cav_template,
                "name" => &citizen.name,
                "password" => &citizen.password,
                "email" => &citizen.email,
                "priv_pass" => &citizen.priv_pass,
                "comment" => &citizen.comment,
                "url" => &citizen.url
            },
        )
        .map_err(|_| ReasonCode::DatabaseError)?;

        Ok(())
    }
}

fn fetch_citizen(row: &Row) -> Result<CitizenQuery, ReasonCode> {
    let id: u32 = database::fetch_int(row, "ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let changed: u32 = database::fetch_int(row, "Changed")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let name: String = database::fetch_string(row, "Name").ok_or(ReasonCode::DatabaseError)?;

    let password: String =
        database::fetch_string(row, "Password").ok_or(ReasonCode::DatabaseError)?;

    let email: String = database::fetch_string(row, "Email").ok_or(ReasonCode::DatabaseError)?;

    let priv_pass: String =
        database::fetch_string(row, "PrivPass").ok_or(ReasonCode::DatabaseError)?;

    let comment: String =
        database::fetch_string(row, "Comment").ok_or(ReasonCode::DatabaseError)?;

    let url: String = database::fetch_string(row, "URL").ok_or(ReasonCode::DatabaseError)?;

    let immigration: u32 = database::fetch_int(row, "Immigration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let expiration: u32 = database::fetch_int(row, "Expiration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_login: u32 = database::fetch_int(row, "LastLogin")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_address: u32 = database::fetch_int(row, "LastAddress")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let total_time: u32 = database::fetch_int(row, "TotalTime")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let bot_limit: u32 = database::fetch_int(row, "BotLimit")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let beta: u32 = database::fetch_int(row, "Beta")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let cav_enabled: u32 = database::fetch_int(row, "CAVEnabled")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let cav_template: u32 = database::fetch_int(row, "CAVTemplate")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let enabled: u32 = database::fetch_int(row, "Enabled")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let privacy: u32 = database::fetch_int(row, "Privacy")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let trial: u32 = database::fetch_int(row, "Trial")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    Ok(CitizenQuery {
        id,
        changed,
        name,
        password,
        email,
        priv_pass,
        comment,
        url,
        immigration,
        expiration,
        last_login,
        last_address,
        total_time,
        bot_limit,
        beta,
        cav_enabled,
        cav_template,
        enabled,
        privacy,
        trial,
    })
}
