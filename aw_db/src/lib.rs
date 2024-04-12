mod config;
mod error;
mod mysql_wrap;
mod row;
mod sqlite_wrap;

use mysql_wrap::mysql_exec;

pub use config::{DatabaseConfig, DatabaseType, MysqlConfig, SqliteConfig};
pub use error::{DatabaseExecError, DatabaseOpenError, DatabaseResult};
pub use row::Row;

pub enum Database {
    External { pool: mysql::Pool },
    Internal { conn: rusqlite::Connection },
}

impl Database {
    pub fn new(config: DatabaseConfig) -> Result<Self, DatabaseOpenError> {
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

        Ok(db)
    }

    #[must_use]
    pub fn exec(
        &self,
        statement: impl AsRef<str>,
        parameters: Vec<String>,
    ) -> DatabaseResult<Vec<Row>> {
        log::trace!(
            "Executing statement {:?} with parameters {:?}",
            statement.as_ref(),
            &parameters
        );

        let res = match &self {
            Database::External { pool } => mysql_exec(pool, statement.as_ref(), parameters.clone()),
            Database::Internal { conn } => {
                sqlite_wrap::sqlite_exec(conn, statement.as_ref(), parameters.clone())
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
