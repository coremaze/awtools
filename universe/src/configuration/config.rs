use std::net::Ipv4Addr;

use super::configurator::run_configurator;
use serde::{Deserialize, Serialize};
use std::path::Path;
const UNIVERSE_CONFIG_PATH: &str = "universe.toml";

/// Struct representing all configurations in the config file.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub universe: UniverseConfig,
    pub mysql: MysqlConfig,
}

/// Configuration section for the universe
#[derive(Deserialize, Serialize, Debug)]
pub struct UniverseConfig {
    pub license_ip: Ipv4Addr,
    pub bind_ip: Ipv4Addr,
    pub port: u16,
    pub user_list: bool,
    pub allow_citizen_changes: bool,
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

impl Config {
    /// Read and (if necessary) generate configuation file.
    pub fn get_interactive() -> Result<Self, String> {
        // Check if config file exists. If not, run configurator.
        // If it does exist, parse it.
        let config_path = Path::new(UNIVERSE_CONFIG_PATH);

        let config = if !config_path.exists() {
            println!(
                "No config file was found at {}. Running configurator.",
                config_path.display()
            );
            run_configurator()
        } else {
            match std::fs::read_to_string(config_path) {
                Ok(contents) => toml::from_str(&contents).map_err(|e| e.to_string())?,
                Err(why) => Err(why.to_string())?,
            }
        };

        config.save();

        Ok(config)
    }

    /// Write configuation to disk.
    pub fn save(&self) {
        let contents = toml::to_string(&self).unwrap_or_default();
        std::fs::write(UNIVERSE_CONFIG_PATH, contents).ok();
    }
}

impl Default for UniverseConfig {
    fn default() -> Self {
        Self {
            license_ip: Ipv4Addr::new(127, 0, 0, 1),
            bind_ip: Ipv4Addr::new(0, 0, 0, 0),
            port: 6670,
            user_list: true,
            allow_citizen_changes: true,
        }
    }
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            hostname: "127.0.0.1".to_string(),
            port: 3306,
            username: "root".to_string(),
            password: "password".to_string(),
            database: "aworld_universe".to_string(),
        }
    }
}
