use std::thread::sleep;
use std::time::Duration;

use aw_core::encoding::latin1_to_string;
use mysql::prelude::Queryable;
use rusqlite::params_from_iter;

use crate::configuration::{DatabaseConfig, DatabaseType, UniverseConfig};

pub use self::attrib::AttribDB;
pub use self::cav::CavDB;
pub use self::citizen::CitizenDB;
pub use self::contact::ContactDB;
pub use self::eject::EjectDB;
pub use self::license::LicenseDB;
pub use self::telegram::TelegramDB;
pub mod attrib;
pub mod cav;
pub mod citizen;
pub mod contact;
pub mod eject;
pub mod license;
pub mod telegram;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseOpenError {
    #[error("Couldn't create the MySQL conneciton pool: {0}")]
    ConnectionPoolFailure(#[from] mysql::Error),
    #[error("Couldn't open the Sqlite database: {0}")]
    SqliteOpenFailure(#[from] rusqlite::Error),
}

pub enum Database {
    External { pool: mysql::Pool },
    Internal { conn: rusqlite::Connection },
}

impl Database {
    pub fn new(
        config: DatabaseConfig,
        universe_config: &UniverseConfig,
    ) -> Result<Self, DatabaseOpenError> {
        let db = match config.database_type {
            DatabaseType::External => {
                let username = &config.mysql_config.username;
                let password = &config.mysql_config.password;
                let hostname = &config.mysql_config.hostname;
                let port = &config.mysql_config.port;
                let database_name = &config.mysql_config.database;
                let uri =
                    format!("mysql://{username}:{password}@{hostname}:{port}/{database_name}");

                Self::External {
                    pool: mysql::Pool::new(uri.as_str())?,
                }
            }
            DatabaseType::Internal => Self::Internal {
                conn: rusqlite::Connection::open(config.sqlite_config.path)?,
            },
        };

        db.init_tables(universe_config);

        Ok(db)
    }

    fn init_tables(&self, universe_config: &UniverseConfig) {
        self.init_attrib(universe_config);
        self.init_citizen();
        self.init_contact();
        self.init_license();
        self.init_telegram();
        self.init_cav();
        self.init_eject();
    }

    fn exec(&self, statement: impl AsRef<str>, parameters: Vec<String>) -> Vec<AWRow> {
        match &self {
            Database::External { pool } => mysql_exec(pool, statement, parameters),
            Database::Internal { conn } => sqlite_exec(conn, statement, parameters),
        }
    }

    pub fn auto_increment_not_null(&self) -> &'static str {
        match &self {
            Database::External { .. } => "NOT NULL AUTO_INCREMENT",
            Database::Internal { .. } => "AUTOINCREMENT NOT NULL",
        }
    }

    pub fn unsigned_str(&self) -> &'static str {
        match &self {
            Database::External { .. } => "unsigned",
            Database::Internal { .. } => "",
        }
    }
}

fn mysql_get_conn(pool: &mysql::Pool) -> mysql::PooledConn {
    loop {
        match pool.get_conn() {
            Ok(c) => {
                return c;
            }
            Err(why) => match why {
                mysql::Error::IoError(_)
                | mysql::Error::CodecError(_)
                | mysql::Error::DriverError(_)
                | mysql::Error::TlsError(_) => {
                    log::error!("Failed to get database connection. Will retry. {why:?}");
                    sleep(Duration::from_secs(1));
                }
                mysql::Error::UrlError(_)
                | mysql::Error::MySqlError(_)
                | mysql::Error::FromValueError(_)
                | mysql::Error::FromRowError(_) => {
                    log::error!("Failed to get database connection. {why:?}");
                    panic!("Database logic issue in mysql_get_conn");
                }
            },
        }
    }
}

fn mysql_exec(
    pool: &mysql::Pool,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Vec<AWRow> {
    let statement = statement.as_ref();
    loop {
        let mut conn = mysql_get_conn(pool);
        let r: mysql::Result<Vec<mysql::Row>> = conn.exec(statement, parameters.clone());
        match r {
            Ok(rows) => {
                return rows
                    .iter()
                    .map(|r| AWRow::MysqlRow(r.clone()))
                    .collect::<Vec<AWRow>>();
            }
            Err(why) => match why {
                mysql::Error::IoError(_)
                | mysql::Error::CodecError(_)
                | mysql::Error::DriverError(_)
                | mysql::Error::TlsError(_) => {
                    log::error!("Failed to execute query. statement: {statement:?}; parameters: {parameters:?}. Will retry. {why:?}");
                    sleep(Duration::from_secs(1));
                }
                mysql::Error::UrlError(_)
                | mysql::Error::MySqlError(_)
                | mysql::Error::FromValueError(_)
                | mysql::Error::FromRowError(_) => {
                    log::error!("Failed to execute query. statement: {statement:?}; parameters: {parameters:?}. {why:?}");
                    panic!("Database logic issue in mysql_exec");
                }
            },
        }
    }
}

fn sqlite_exec(
    conn: &rusqlite::Connection,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Vec<AWRow> {
    let statement = statement.as_ref();
    let mut stmt = match conn.prepare(statement) {
        Ok(stmt) => stmt,
        Err(why) => {
            log::error!("Failed to prepare statement: {statement:?}. {why:?}");
            panic!("Database logic issue in sqlite_exec");
        }
    };
    let column_names = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    let mut rows = loop {
        match stmt.query(params_from_iter(parameters.iter())) {
            Ok(rows) => break rows,
            Err(why) => match why {
                rusqlite::Error::SqliteFailure(_, _) => {
                    log::error!("Failed to execute query. statement: {statement:?}; parameters: {parameters:?}. Will retry. {why:?}");
                    sleep(Duration::from_secs(1));
                }

                rusqlite::Error::SqliteSingleThreadedMode
                | rusqlite::Error::StatementChangedRows(_)
                | rusqlite::Error::QueryReturnedNoRows
                | rusqlite::Error::FromSqlConversionFailure(_, _, _)
                | rusqlite::Error::IntegralValueOutOfRange(_, _)
                | rusqlite::Error::Utf8Error(_)
                | rusqlite::Error::NulError(_)
                | rusqlite::Error::InvalidParameterName(_)
                | rusqlite::Error::InvalidPath(_)
                | rusqlite::Error::ExecuteReturnedResults
                | rusqlite::Error::InvalidColumnIndex(_)
                | rusqlite::Error::InvalidColumnName(_)
                | rusqlite::Error::InvalidColumnType(_, _, _)
                | rusqlite::Error::ToSqlConversionFailure(_)
                | rusqlite::Error::InvalidQuery
                | rusqlite::Error::MultipleStatement
                | rusqlite::Error::InvalidParameterCount(_, _) => {
                    log::error!("Failed to execute query. statement: {statement:?}; parameters: {parameters:?}. {why:?}");
                    panic!("Database logic issue in sqlite_exec");
                }
                _ => {
                    log::error!(
                        "Failed to execute query. statement: {statement:?}; parameters: {parameters:?}. Unknown error: {why:?}"
                    );
                    panic!("Database logic issue in sqlite_exec");
                }
            },
        }
    };

    let mut rows_vec = Vec::<AWRow>::new();
    while let Ok(Some(row)) = rows.next() {
        let mut row_vec = Vec::<SqliteValue>::new();
        for i in 0..column_names.len() {
            if let Ok(num) = row.get::<usize, i64>(i) {
                row_vec.push(SqliteValue::Int(num))
            } else if let Ok(b) = row.get::<usize, String>(i) {
                row_vec.push(SqliteValue::String(b))
            } else {
                row_vec.push(SqliteValue::None)
            }
        }
        rows_vec.push(AWRow::SqliteRow {
            column_names: column_names.clone(),
            row: row_vec,
        });
    }
    rows_vec
}

#[derive(Debug)]
pub enum AWRow {
    MysqlRow(mysql::Row),
    SqliteRow {
        column_names: Vec<String>,
        row: Vec<SqliteValue>,
    },
}

#[derive(Debug)]
pub enum SqliteValue {
    None,
    Int(i64),
    String(String),
}

impl AWRow {
    pub fn fetch_int(&self, name: &str) -> Option<i64> {
        match &self {
            AWRow::MysqlRow(row) => match row.get::<mysql::Value, _>(name) {
                Some(mysql::Value::Int(x)) => Some(x),
                _ => None,
            },
            AWRow::SqliteRow { column_names, row } => match column_names
                .iter()
                .enumerate()
                .find(|(_, cname)| *cname == name)
                .and_then(|(i, _)| row.get(i))
            {
                Some(SqliteValue::Int(x)) => Some(*x),
                _ => None,
            },
        }
    }

    pub fn fetch_string(&self, name: &str) -> Option<String> {
        match &self {
            AWRow::MysqlRow(row) => match row.get::<mysql::Value, _>(name) {
                Some(mysql::Value::Bytes(x)) => Some(latin1_to_string(&x)),
                _ => None,
            },
            AWRow::SqliteRow { column_names, row } => match column_names
                .iter()
                .enumerate()
                .find(|(_, cname)| *cname == name)
                .and_then(|(i, _)| row.get(i))
            {
                Some(SqliteValue::String(x)) => Some(x.clone()),
                _ => None,
            },
        }
    }
}

#[macro_export]
macro_rules! aw_params {
    ($($param:expr),*) => {
        {
            let mut params: Vec<String> = Vec::new();
            $(
                params.push(format!("{}", $param));
            )*
            params
        }
    };
}
