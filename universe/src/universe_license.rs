use aw_core::*;
use std::net::SocketAddrV4;

/// Generates licenses for sending to clients.
///
/// ActiveWorlds relies on providing Universe owners with a license containing
/// information about the Universe's attributes and capabilities which is then
/// encrypted with a private RSA key held by ActiveWorlds, Inc. The most
/// important of this information is the IP address and port of the Universe,
/// since clients will decrypt this license with the public RSA key and then
/// check to see if they are the IP and port being connected to. If this
/// process fails, the client will refuse to connect.
///
/// Since the RSA key length used for this process was only 512 bits, it was
/// possible to factor n back into primes p and q in order to derive the
/// private key. We use this in order to allow unmodified ActiveWorlds clients
/// to connect.
///
/// This also provides compatibility with the Vortex ActiveWorlds 5.1 client.
pub struct LicenseGenerator {
    ip: SocketAddrV4,
}

impl LicenseGenerator {
    pub fn new(ip: &SocketAddrV4) -> Self {
        Self { ip: *ip }
    }

    pub fn create_license_data(&self, browser_build: i32) -> Vec<u8> {
        let key = match browser_build {
            /* Vortex 5.1 */ 1217 => include_bytes!("keys/vortex.priv"),
            /* Vortex 5.1 SDK */ 85 => include_bytes!("keys/vortex.priv"),
            /* Miuchiz R7 */ 2007 => include_bytes!("keys/vortex.priv"),
            /* Regular AW */ _ => include_bytes!("keys/aw.priv"),
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
