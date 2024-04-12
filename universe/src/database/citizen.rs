use crate::aw_params;

use super::{AWRow, Database, DatabaseResult};

use std::time::{SystemTime, UNIX_EPOCH};

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
    fn init_citizen(&self) -> DatabaseResult<()>;
    fn citizen_by_name(&self, name: &str) -> DatabaseResult<Option<CitizenQuery>>;
    fn citizen_by_number(&self, citizen_id: u32) -> DatabaseResult<Option<CitizenQuery>>;
    fn citizen_add(&self, citizen: &CitizenQuery) -> DatabaseResult<()>;
    fn citizen_add_next(&self, citizen: CitizenQuery) -> DatabaseResult<()>;
    fn citizen_change(&self, citizen: &CitizenQuery) -> DatabaseResult<()>;
}

impl CitizenDB for Database {
    fn init_citizen(&self) -> DatabaseResult<()> {
        let auto_increment_not_null = self.auto_increment_not_null();
        let r = self.exec(
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

        if r.is_err() {
            return DatabaseResult::DatabaseError;
        }

        // Create default Administrator account if one doesn't exist yet
        match self.citizen_by_number(1) {
            DatabaseResult::Ok(Some(_)) => { /* Administrator exists, no work to be done */ }
            DatabaseResult::Ok(None) => {
                // Administrator does not exist yet - create it
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

                if self.citizen_add(&admin).is_err() {
                    eprintln!("Failed to create citizen #1");
                    return DatabaseResult::DatabaseError;
                }
                println!("Citizen #1 created as {} / {}", admin.name, admin.password);
            }
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        }

        DatabaseResult::Ok(())
    }

    fn citizen_by_name(&self, name: &str) -> DatabaseResult<Option<CitizenQuery>> {
        let rows = match self.exec("SELECT * FROM awu_citizen WHERE Name=?", aw_params!(name)) {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.len() > 1 {
            log::error!("There exist multiple users with the name {name:?}");
            log::error!("{rows:?}");
            return DatabaseResult::DatabaseError;
        }

        let Some(user) = rows.first() else {
            // There were no users by this name
            return DatabaseResult::Ok(None);
        };

        match fetch_citizen(user) {
            DatabaseResult::Ok(user) => DatabaseResult::Ok(Some(user)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn citizen_by_number(&self, citizen_id: u32) -> DatabaseResult<Option<CitizenQuery>> {
        let rows = match self.exec(
            r"SELECT * FROM awu_citizen WHERE ID=?",
            aw_params!(citizen_id),
        ) {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.len() > 1 {
            log::error!("There exist multiple users with the citizen id {citizen_id:?}");
            log::error!("{rows:?}");
            return DatabaseResult::DatabaseError;
        }

        let Some(user) = rows.first() else {
            // There were no users by this number
            return DatabaseResult::Ok(None);
        };

        match fetch_citizen(user) {
            DatabaseResult::Ok(user) => DatabaseResult::Ok(Some(user)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn citizen_add(&self, citizen: &CitizenQuery) -> DatabaseResult<()> {
        let r = self.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn citizen_add_next(&self, mut citizen: CitizenQuery) -> DatabaseResult<()> {
        // Get the next unused ID from the database
        let rows = match self.exec("SELECT MAX(ID) + 1 AS ID FROM awu_citizen", vec![]) {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };
        let next_id = rows.first();

        let id: u32 = match next_id {
            Some(row) => match row.fetch_int("ID").map(u32::try_from) {
                Some(Ok(x)) => x,
                _ => return DatabaseResult::DatabaseError,
            },
            None => 1,
        };

        citizen.id = id;

        let r = self.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn citizen_change(&self, citizen: &CitizenQuery) -> DatabaseResult<()> {
        let r = self.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}

fn fetch_citizen(row: &AWRow) -> DatabaseResult<CitizenQuery> {
    let id = match row.fetch_int("ID").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let changed = match row.fetch_int("Changed").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let name = match row.fetch_string("Name") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let password = match row.fetch_string("Password") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let email = match row.fetch_string("Email") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let priv_pass = match row.fetch_string("PrivPass") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let comment = match row.fetch_string("Comment") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let url = match row.fetch_string("URL") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let immigration = match row.fetch_int("Immigration").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let expiration = match row.fetch_int("Expiration").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let last_login = match row.fetch_int("LastLogin").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    // It is not unexpected for LastAddress to end up in the database as a negative number.
    let last_address = match row.fetch_int("LastAddress").map(|x| x as u32) {
        Some(x) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let total_time = match row.fetch_int("TotalTime").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let bot_limit = match row.fetch_int("BotLimit").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let beta = match row.fetch_int("Beta").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let cav_enabled = match row.fetch_int("CAVEnabled").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let cav_template = match row.fetch_int("CAVTemplate").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let enabled = match row.fetch_int("Enabled").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let privacy = match row.fetch_int("Privacy").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let trial = match row.fetch_int("Trial").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    DatabaseResult::Ok(CitizenQuery {
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
