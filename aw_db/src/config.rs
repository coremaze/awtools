use serde::{Deserialize, Serialize};

/// Internal (sqlite, local, on-disk, self contained) database or external (server) database
#[derive(Deserialize, Serialize, Debug, Default)]
pub enum DatabaseType {
    External,
    #[default]
    Internal,
}

/// Config for sqlite, the local database solution
#[derive(Deserialize, Serialize, Debug)]
pub struct SqliteConfig {
    pub path: String,
}

/// Configuation section for the mysql connection
#[derive(Deserialize, Serialize, Debug)]
pub struct MysqlConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

/// Configuration for the database, whether external or internal
#[derive(Deserialize, Serialize, Debug)]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub mysql_config: MysqlConfig,
    pub sqlite_config: SqliteConfig,
}
