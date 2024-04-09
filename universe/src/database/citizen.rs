use crate::aw_params;

use super::{AWRow, Database};
use aw_core::ReasonCode;

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
    fn citizen_by_name(&self, name: &str) -> Result<CitizenQuery, ReasonCode>;
    fn citizen_by_number(&self, citizen_id: u32) -> Result<CitizenQuery, ReasonCode>;
    fn citizen_add(&self, citizen: &CitizenQuery) -> Result<(), ReasonCode>;
    fn citizen_add_next(&self, citizen: CitizenQuery) -> Result<(), ReasonCode>;
    fn citizen_change(&self, citizen: &CitizenQuery) -> Result<(), ReasonCode>;
}

impl CitizenDB for Database {
    fn init_citizen(&self) {
        let auto_increment_not_null = self.auto_increment_not_null();
        self.exec(
            format!(
                r"CREATE TABLE IF NOT EXISTS awu_citizen ( 
            ID INTEGER PRIMARY KEY {auto_increment_not_null}, 
            Changed tinyint(1) NOT NULL default '0', 
            Name varchar(255) NOT NULL default '', 
            Password varchar(255) NOT NULL default '', 
            Email varchar(255) NOT NULL default '', 
            PrivPass varchar(255) NOT NULL default '', 
            Comment varchar(255) NOT NULL default '', 
            URL varchar(255) NOT NULL default '', 
            Immigration INTEGER NOT NULL default '0', 
            Expiration INTEGER NOT NULL default '0', 
            LastLogin INTEGER NOT NULL default '0', 
            LastAddress INTEGER NOT NULL default '0', 
            TotalTime INTEGER NOT NULL default '0', 
            BotLimit INTEGER NOT NULL default '0', 
            Beta tinyint(1) NOT NULL default '0', 
            CAVEnabled tinyint(1) NOT NULL default '0', 
            CAVTemplate INTEGER NOT NULL default '0', 
            Enabled tinyint(1) NOT NULL default '1', 
            Privacy INTEGER NOT NULL default '0', 
            Trial tinyint(1) NOT NULL default '0'
        );"
            ),
            vec![],
        );

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

    fn citizen_by_name(&self, name: &str) -> Result<CitizenQuery, ReasonCode> {
        let rows = self.exec("SELECT * FROM awu_citizen WHERE Name=?", aw_params!(name));

        if rows.len() > 1 {
            return Err(ReasonCode::DatabaseError);
        }

        if let Some(user) = rows.first() {
            fetch_citizen(user)
        } else {
            Err(ReasonCode::DatabaseError)
        }
    }

    fn citizen_by_number(&self, citizen_id: u32) -> Result<CitizenQuery, ReasonCode> {
        let rows = self.exec(
            r"SELECT * FROM awu_citizen WHERE ID=?",
            aw_params!(citizen_id),
        );

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
        self.exec(
            r"INSERT INTO awu_citizen(
                ID, Immigration, Expiration, LastLogin, LastAddress, TotalTime, 
                BotLimit, Beta, Enabled, Trial, Privacy, CAVEnabled, CAVTemplate, 
                Name, Password, Email, PrivPass, Comment, URL) 
            VALUES(?, ?, ?, ?, ?, ?, 
                ?, ?, ?, ?, ?, ?, ?, 
                ?, ?, ?, ?, ?, ?)",
            aw_params!(
                citizen.id,
                citizen.immigration,
                citizen.expiration,
                citizen.last_login,
                citizen.last_address,
                citizen.total_time,
                citizen.bot_limit,
                citizen.beta,
                citizen.enabled,
                citizen.trial,
                citizen.privacy,
                citizen.cav_enabled,
                citizen.cav_template,
                &citizen.name,
                &citizen.password,
                &citizen.email,
                &citizen.priv_pass,
                &citizen.comment,
                &citizen.url
            ),
        );

        Ok(())
    }

    fn citizen_add_next(&self, mut citizen: CitizenQuery) -> Result<(), ReasonCode> {
        // Get the next unused ID from the database
        let rows = self.exec("SELECT MAX(ID) + 1 AS ID FROM awu_citizen", vec![]);
        let next_id = rows.first();

        let id: u32 = match next_id {
            Some(row) => row
                .fetch_int("ID")
                .ok_or(ReasonCode::DatabaseError)?
                .try_into()
                .map_err(|_| ReasonCode::DatabaseError)?,
            None => 1,
        };

        citizen.id = id;

        self.exec(
            r"INSERT INTO awu_citizen(
                ID, Immigration, Expiration, LastLogin, LastAddress, TotalTime, 
                BotLimit, Beta, Enabled, Trial, Privacy, CAVEnabled, CAVTemplate, 
                Name, Password, Email, PrivPass, Comment, URL) 
            VALUES(?, ?, ?, ?, ?, ?, 
                ?, ?, ?, ?, ?, ?, ?, 
                ?, ?, ?, ?, ?, ?)",
            aw_params! {
                citizen.id,
                citizen.immigration,
                citizen.expiration,
                citizen.last_login,
                citizen.last_address,
                citizen.total_time,
                citizen.bot_limit,
                citizen.beta,
                citizen.enabled,
                citizen.trial,
                citizen.privacy,
                citizen.cav_enabled,
                citizen.cav_template,
                &citizen.name,
                &citizen.password,
                &citizen.email,
                &citizen.priv_pass,
                &citizen.comment,
                &citizen.url
            },
        );

        Ok(())
    }

    fn citizen_change(&self, citizen: &CitizenQuery) -> Result<(), ReasonCode> {
        self.exec(
            r"UPDATE awu_citizen SET Changed=NOT Changed,
                Immigration=?, Expiration=?, LastLogin=?, 
                LastAddress=?, TotalTime=?, BotLimit=?, 
                Beta=?, Enabled=?, Trial=?, Privacy=?, 
                CAVEnabled=?, CAVTemplate=?, Name=?, 
                Password=?, Email=?, PrivPass=?, 
                Comment=?, URL=?
                WHERE ID=?;",
            aw_params! {
                citizen.immigration,
                citizen.expiration,
                citizen.last_login,
                citizen.last_address,
                citizen.total_time,
                citizen.bot_limit,
                citizen.beta,
                citizen.enabled,
                citizen.trial,
                citizen.privacy,
                citizen.cav_enabled,
                citizen.cav_template,
                &citizen.name,
                &citizen.password,
                &citizen.email,
                &citizen.priv_pass,
                &citizen.comment,
                &citizen.url,
                citizen.id
            },
        );

        Ok(())
    }
}

fn fetch_citizen(row: &AWRow) -> Result<CitizenQuery, ReasonCode> {
    let id: u32 = row
        .fetch_int("ID")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let changed: u32 = row
        .fetch_int("Changed")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let name: String = row.fetch_string("Name").ok_or(ReasonCode::DatabaseError)?;

    let password: String = row
        .fetch_string("Password")
        .ok_or(ReasonCode::DatabaseError)?;

    let email: String = row.fetch_string("Email").ok_or(ReasonCode::DatabaseError)?;

    let priv_pass: String = row
        .fetch_string("PrivPass")
        .ok_or(ReasonCode::DatabaseError)?;

    let comment: String = row
        .fetch_string("Comment")
        .ok_or(ReasonCode::DatabaseError)?;

    let url: String = row.fetch_string("URL").ok_or(ReasonCode::DatabaseError)?;

    let immigration: u32 = row
        .fetch_int("Immigration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let expiration: u32 = row
        .fetch_int("Expiration")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let last_login: u32 = row
        .fetch_int("LastLogin")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    // It is not unexpected for LastAddress to end up in the database as a negative number.
    let last_address: u32 = row
        .fetch_int("LastAddress")
        .ok_or(ReasonCode::DatabaseError)? as u32;

    let total_time: u32 = row
        .fetch_int("TotalTime")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let bot_limit: u32 = row
        .fetch_int("BotLimit")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let beta: u32 = row
        .fetch_int("Beta")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let cav_enabled: u32 = row
        .fetch_int("CAVEnabled")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let cav_template: u32 = row
        .fetch_int("CAVTemplate")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let enabled: u32 = row
        .fetch_int("Enabled")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let privacy: u32 = row
        .fetch_int("Privacy")
        .ok_or(ReasonCode::DatabaseError)?
        .try_into()
        .map_err(|_| ReasonCode::DatabaseError)?;

    let trial: u32 = row
        .fetch_int("Trial")
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
