use mysql::*;

use crate::config::MysqlConfig;

use self::attrib::AttribDB;
use self::cav::CavDB;
use self::citizen::CitizenDB;
use self::contact::ContactDB;
use self::eject::EjectDB;
use self::license::LicenseDB;
use self::telegram::TelegramDB;
pub mod attrib;
pub mod cav;
pub mod citizen;
pub mod contact;
pub mod eject;
pub mod license;
pub mod telegram;

type Result<T, E> = core::result::Result<T, E>;
use std::error::Error;
pub struct Database {
    pool: Pool,
    config: MysqlConfig,
}

impl Database {
    pub fn new(config: MysqlConfig) -> Result<Self, String> {
        let username = &config.username;
        let password = &config.password;
        let hostname = &config.hostname;
        let port = &config.port;
        let database_name = &config.database;
        let uri = format!("mysql://{username}:{password}@{hostname}:{port}/{database_name}");
        let pool = Pool::new(uri.as_str())
            .map_err(|err| format!("Could not create database connection pool: {err}"))?;

        let db = Self { pool, config };

        db.init_tables();

        Ok(db)
    }

    pub fn conn(&self) -> Result<PooledConn, Box<dyn Error>> {
        Ok(self.pool.get_conn()?)
    }

    fn init_tables(&self) {
        self.init_attrib();
        self.init_citizen();
        self.init_contact();
        self.init_license();
        self.init_telegram();
        self.init_cav();
        self.init_eject();
    }
}

pub fn fetch_int(row: &Row, name: &str) -> Option<i64> {
    for column in row.columns_ref() {
        let column_value = &row[column.name_str().as_ref()];
        let column_name = column.name_str().to_string();
        if column_name == name {
            match column_value {
                Value::Int(x) => {
                    return Some(*x);
                }
                _ => {
                    return None;
                }
            }
        }
    }
    None
}

pub fn fetch_string(row: &Row, name: &str) -> Option<String> {
    for column in row.columns_ref() {
        let column_value = &row[column.name_str().as_ref()];
        let column_name = column.name_str().to_string();
        if column_name == name {
            match column_value {
                Value::Bytes(x) => {
                    return Some(aw_core::encoding::latin1_to_string(x));
                }
                _ => {
                    return None;
                }
            }
        }
    }
    None
}
