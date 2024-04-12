use std::time::{SystemTime, UNIX_EPOCH};

use aw_db::{aw_params, DatabaseResult, Row};

use super::UniverseDatabase;

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
    fn init_license(&self) -> DatabaseResult<()>;
    fn license_by_name(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>>;
    fn license_add(&self, lic: &LicenseQuery) -> DatabaseResult<()>;
    fn license_next(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>>;
    fn license_prev(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>>;
    fn license_change(&self, lic: &LicenseQuery) -> DatabaseResult<()>;
}

impl LicenseDB for UniverseDatabase {
    fn init_license(&self) -> DatabaseResult<()> {
        let auto_increment_not_null = self.db.auto_increment_not_null();
        // "Range" has been changed to "WorldSize" because range is now a keyword.
        let r = self.db.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn license_by_name(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_license WHERE Name=?",
            aw_params! {
                name
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.len() > 1 {
            log::error!("There is more than one license for world name {name:?}");
            log::error!("{rows:?}");
            return DatabaseResult::DatabaseError;
        }

        let Some(row) = rows.first() else {
            return DatabaseResult::Ok(None);
        };

        match fetch_license(row) {
            DatabaseResult::Ok(lic) => DatabaseResult::Ok(Some(lic)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn license_add(&self, lic: &LicenseQuery) -> DatabaseResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current time is before the unix epoch.")
            .as_secs();

        let r = self.db.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn license_next(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_license WHERE Name>? ORDER BY Name LIMIT 1",
            aw_params! {
                name
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.len() > 1 {
            return DatabaseResult::DatabaseError;
        }

        let Some(row) = rows.first() else {
            return DatabaseResult::Ok(None);
        };

        match fetch_license(row) {
            DatabaseResult::Ok(lic) => DatabaseResult::Ok(Some(lic)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn license_prev(&self, name: &str) -> DatabaseResult<Option<LicenseQuery>> {
        let r = self.db.exec(
            r"SELECT * FROM awu_license WHERE Name<? ORDER BY Name DESC LIMIT 1",
            aw_params! {
                name
            },
        );

        let rows = match r {
            DatabaseResult::Ok(rows) => rows,
            DatabaseResult::DatabaseError => return DatabaseResult::DatabaseError,
        };

        if rows.len() > 1 {
            return DatabaseResult::DatabaseError;
        }

        let Some(row) = rows.first() else {
            return DatabaseResult::Ok(None);
        };

        match fetch_license(row) {
            DatabaseResult::Ok(lic) => DatabaseResult::Ok(Some(lic)),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }

    fn license_change(&self, lic: &LicenseQuery) -> DatabaseResult<()> {
        let r = self.db.exec(
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

        match r {
            DatabaseResult::Ok(_) => DatabaseResult::Ok(()),
            DatabaseResult::DatabaseError => DatabaseResult::DatabaseError,
        }
    }
}

fn fetch_license(row: &Row) -> DatabaseResult<LicenseQuery> {
    let id = match row.fetch_int("ID").map(u32::try_from) {
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

    let comment = match row.fetch_string("Comment") {
        Some(x) => x,
        None => return DatabaseResult::DatabaseError,
    };

    let creation = match row.fetch_int("Creation").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let expiration = match row.fetch_int("Expiration").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let last_start = match row.fetch_int("LastStart").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    // It is not unexpected for LastAddress to end up in the database as a negative number.
    let last_address = match row.fetch_int("LastAddress").map(|x| x as u32) {
        Some(x) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let users = match row.fetch_int("Users").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let world_size = match row.fetch_int("WorldSize").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let hidden = match row.fetch_int("Hidden").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let changed = match row.fetch_int("Changed").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let tourists = match row.fetch_int("Tourists").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let voip = match row.fetch_int("Voip").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    let plugins = match row.fetch_int("Plugins").map(u32::try_from) {
        Some(Ok(x)) => x,
        _ => return DatabaseResult::DatabaseError,
    };

    DatabaseResult::Ok(LicenseQuery {
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
