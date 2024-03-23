use crate::crypt_rsa::{AWCryptRSA, RSAKey};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::net::Ipv4Addr;

pub struct AWRegLic {
    rsa: AWCryptRSA,
}

impl AWRegLic {
    pub fn new(rsa: AWCryptRSA) -> Self {
        Self { rsa }
    }

    pub fn code_process_base64(
        &mut self,
        source: &str,
        key_type: RSAKey,
    ) -> Result<AWRegLicData, String> {
        match base64::decode(source) {
            Ok(decoded) => self.code_process_binary(&decoded, key_type),
            Err(_) => Err("Failed to decode code.".to_string()),
        }
    }

    pub fn code_process_binary(
        &mut self,
        source: &[u8],
        key_type: RSAKey,
    ) -> Result<AWRegLicData, String> {
        let decrypted = self
            .rsa
            .decrypt(source, key_type)
            .map_err(|_| "Failed to decrypt code.".to_string())?;
        AWRegLicData::decode(&decrypted)
    }

    pub fn code_generate_binary(
        &mut self,
        data: &AWRegLicData,
        key_type: RSAKey,
    ) -> Result<Vec<u8>, String> {
        self.rsa
            .encrypt(&data.encode(), key_type)
            .map_err(|_| "Failed to encrypt code.".to_string())
    }

    pub fn code_generate_base64(
        &mut self,
        data: &AWRegLicData,
        key_type: RSAKey,
    ) -> Result<String, String> {
        let bin = self.code_generate_binary(data, key_type)?;
        Ok(base64::encode(bin))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AWRegLicData {
    license_version: u32,
    ip_address: [u8; 4],
    port: u32,
    land_limit: u32,
    max_users: u32,
    world_limit: u32,
    expiration_time: i32,
    major_version: u16,
    minor_version: u16,
    name: [u8; 0x20],
    can_have_bots: u32,
}

impl AWRegLicData {
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        bincode::deserialize(data).map_err(|_| "Failed to deserialize license.".to_string())
    }

    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Could not encode AWRegLic.")
    }

    pub fn get_license_version(&self) -> u32 {
        self.license_version
    }

    pub fn get_ip_address(&self) -> Ipv4Addr {
        let a = &self.ip_address;
        Ipv4Addr::new(a[0], a[1], a[2], a[3])
    }

    pub fn get_port(&self) -> u32 {
        self.port
    }

    pub fn get_land_limit(&self) -> u32 {
        self.land_limit
    }

    pub fn get_max_users(&self) -> u32 {
        self.max_users
    }

    pub fn get_world_limit(&self) -> u32 {
        self.world_limit
    }

    pub fn get_expiration_time(&self) -> i32 {
        self.expiration_time
    }

    pub fn get_major_version(&self) -> u16 {
        self.major_version
    }

    pub fn get_minor_version(&self) -> u16 {
        self.minor_version
    }

    pub fn get_name(&self) -> String {
        String::from_utf8_lossy(&self.name).to_string()
    }

    pub fn get_can_have_bots(&self) -> bool {
        self.can_have_bots != 0
    }

    pub fn set_license_version(mut self, license_version: u32) -> Self {
        self.license_version = license_version;
        self
    }

    pub fn set_ip_address(mut self, ip_address: &Ipv4Addr) -> Self {
        self.ip_address = ip_address.octets();
        self
    }

    pub fn set_port(mut self, port: u32) -> Self {
        self.port = port;
        self
    }

    pub fn set_land_limit(mut self, land_limit: u32) -> Self {
        self.land_limit = land_limit;
        self
    }

    pub fn set_max_users(mut self, max_users: u32) -> Self {
        self.max_users = max_users;
        self
    }

    pub fn set_world_limit(mut self, world_limit: u32) -> Self {
        self.world_limit = world_limit;
        self
    }

    pub fn set_expiration_time(mut self, expiration_time: i32) -> Self {
        self.expiration_time = expiration_time;
        self
    }

    pub fn set_major_version(mut self, major_version: u16) -> Self {
        self.major_version = major_version;
        self
    }

    pub fn set_minor_version(mut self, minor_version: u16) -> Self {
        self.minor_version = minor_version;
        self
    }

    pub fn set_name(mut self, name: &str) -> Self {
        let name_bytes = name.as_bytes();
        self.name = [0u8; 0x20];

        for (target, source) in self.name.iter_mut().zip(name_bytes) {
            *target = *source;
        }
        self
    }

    pub fn set_can_have_bots(mut self, can_have_bots: bool) -> Self {
        self.can_have_bots = can_have_bots.into();
        self
    }
}

impl Default for AWRegLicData {
    fn default() -> Self {
        Self {
            license_version: 1,
            ip_address: Ipv4Addr::new(127, 0, 0, 1).octets(),
            port: 6670,
            land_limit: 0,
            max_users: 0,
            world_limit: 0,
            expiration_time: i32::MAX,
            major_version: 5,
            minor_version: 1,
            name: [0; 0x20],
            can_have_bots: true.into(),
        }
    }
}

impl Display for AWRegLicData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "license_version: {}\n",
            self.get_license_version()
        ))?;
        f.write_fmt(format_args!("ip_address: {}\n", self.get_ip_address()))?;
        f.write_fmt(format_args!("port: {}\n", self.get_port()))?;
        f.write_fmt(format_args!("land_limit: {}\n", self.get_land_limit()))?;
        f.write_fmt(format_args!("max_users: {}\n", self.get_max_users()))?;
        f.write_fmt(format_args!("world_limit: {}\n", self.get_world_limit()))?;
        f.write_fmt(format_args!(
            "expiration_time: {}\n",
            self.get_expiration_time()
        ))?;
        f.write_fmt(format_args!(
            "major_version: {}\n",
            self.get_major_version()
        ))?;
        f.write_fmt(format_args!(
            "minor_version: {}\n",
            self.get_minor_version()
        ))?;
        f.write_fmt(format_args!("name: {}\n", self.get_name()))?;
        f.write_fmt(format_args!("can_have_bots: {}", self.get_can_have_bots()))?;
        Ok(())
    }
}
