use aw_core::*;

use crate::client::UserInfo;
use std::net::SocketAddrV4;

pub struct LicenseGenerator {
    ip: SocketAddrV4,
}

impl LicenseGenerator {
    pub fn new(ip: &SocketAddrV4) -> Self {
        Self { ip: *ip }
    }

    pub fn create_license_data(&self, user: &UserInfo) -> Vec<u8> {
        let key = match user.build_version {
            /* Vortex 5.1 or */ Some(1217) |
            /* Miuchiz R7    */ Some(2007)  => include_bytes!("keys/vortex.priv"),
            /* Regular AW    */ _ => include_bytes!("keys/aw.priv"), 
        };

        let mut rsa = AWCryptRSA::default();
        rsa.decode_private_key(key)
            .expect("Couldn't decode RSA key.");

        let mut reg_lic = AWRegLic::new(rsa);
        let reg_lic_data = AWRegLicData::default()
            .set_ip_address(self.ip.ip())
            .set_port(self.ip.port() as u32)
            .set_name("aw")
            .set_expiration_time(i32::MAX);

        reg_lic
            .code_generate_binary(&reg_lic_data, RSAKey::Private)
            .expect("Could not generate license")
    }
}
