use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
const UNIVERSE_CONFIG_PATH: &str = "universe.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct UniverseConfig {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl UniverseConfig {
    pub fn get() -> Result<Self, String> {
        let config: Self = match std::fs::read_to_string(UNIVERSE_CONFIG_PATH) {
            Ok(contents) => toml::from_str(&contents).map_err(|e| e.to_string())?,
            Err(_) => UniverseConfig::default(),
        };

        config.save();

        Ok(config)
    }

    pub fn save(&self) {
        let contents = toml::to_string(&self).unwrap_or_default();
        std::fs::write(UNIVERSE_CONFIG_PATH, contents).ok();
    }
}

impl Default for UniverseConfig {
    fn default() -> Self {
        Self {
            ip: Ipv4Addr::new(127, 0, 0, 1),
            port: 6670,
        }
    }
}
