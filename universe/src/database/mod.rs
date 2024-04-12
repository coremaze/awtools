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

#[derive(thiserror::Error, Debug)]
pub enum DatabaseExecError {
    #[error("Encountered mysql error {0}")]
    MysqlError(#[from] mysql::Error),
    #[error("Encountered sqlite error {0}")]
    SqliteError(#[from] rusqlite::Error),
}

#[derive(Debug, Copy, Clone)]
pub enum DatabaseResult<T> {
    Ok(T),
    DatabaseError,
}

impl<T> DatabaseResult<T> {
    pub fn is_err(&self) -> bool {
        match self {
            DatabaseResult::Ok(_) => false,
            DatabaseResult::DatabaseError => true,
        }
    }
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

    #[must_use]
    fn exec(
        &self,
        statement: impl AsRef<str>,
        parameters: Vec<String>,
    ) -> DatabaseResult<Vec<AWRow>> {
        let res = match &self {
            Database::External { pool } => mysql_exec(pool, statement.as_ref(), parameters.clone()),
            Database::Internal { conn } => {
                sqlite_exec(conn, statement.as_ref(), parameters.clone())
            }
        };

        match res {
            Ok(rows) => DatabaseResult::Ok(rows),
            Err(why) => {
                log::error!("A database execution failed: {why:?}");
                log::error!("Statement: {:?}", statement.as_ref());
                log::error!("Parameters: {parameters:?}");

                DatabaseResult::DatabaseError
            }
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

fn mysql_exec(
    pool: &mysql::Pool,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Result<Vec<AWRow>, DatabaseExecError> {
    let mut conn = pool.get_conn()?;
    let r: mysql::Result<Vec<mysql::Row>> = conn.exec(statement.as_ref(), parameters.clone());

    Ok(r?
        .iter()
        .map(|r| AWRow::MysqlRow(r.clone()))
        .collect::<Vec<AWRow>>())
}

fn sqlite_exec(
    conn: &rusqlite::Connection,
    statement: impl AsRef<str>,
    parameters: Vec<String>,
) -> Result<Vec<AWRow>, DatabaseExecError> {
    let mut stmt = conn.prepare(statement.as_ref())?;

    let column_names = stmt
        .column_names()
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    let mut rows = stmt.query(params_from_iter(parameters.iter()))?;

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
    Ok(rows_vec)
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
