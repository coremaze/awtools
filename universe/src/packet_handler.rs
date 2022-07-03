use crate::license::LicenseGenerator;
use crate::{
    attributes,
    client::{Client, ClientManager, ClientType},
};
use aw_core::*;
use num_traits::FromPrimitive;

pub fn public_key_request(client: &Client) {
    let key = client
        .rsa
        .encode_public_key()
        .expect("Public key was missing.");

    let mut packet = AWPacket::new(PacketType::PublicKeyResponse);
    packet.add_var(AWPacketVar::Data(VarID::EncryptionKey, key));
    client.connection.send(packet);
}

pub fn stream_key_response(client: &Client, packet: &AWPacket) {
    if let Some(encrypted_a4_key) = packet.get_data(VarID::EncryptionKey) {
        if let Ok(a4_key) = client.rsa.decrypt_private(&encrypted_a4_key) {
            client.connection.set_recv_key(&a4_key);
            attributes::send_attributes(client);
        }
    }
}

pub fn public_key_response(client: &Client, packet: &AWPacket) {
    if let Some(rsa_key_bytes) = packet.get_data(VarID::EncryptionKey) {
        // Decode their public key
        let mut public_rsa = AWCryptRSA::default();
        public_rsa.randomize();
        if public_rsa.decode_public_key(&rsa_key_bytes).is_err() {
            return;
        }

        // Encrypt our RC4 key using the client's RSA key
        match public_rsa.encrypt_public(&client.connection.get_send_key()) {
            Ok(encrypted_a4) => {
                let mut response = AWPacket::new(PacketType::StreamKeyResponse);
                response.add_var(AWPacketVar::Data(VarID::EncryptionKey, encrypted_a4));
                client.connection.send(response);
                client.connection.encrypt_data(true);
            }
            Err(e) => {
                println!("Failed to encrypt: {e:?}");
            }
        }
    }
}

pub fn login(
    client: &Client,
    packet: &AWPacket,
    client_manager: &ClientManager,
    license_generator: &LicenseGenerator,
) {
    let _client_version = packet.get_int(VarID::BrowserVersion);
    let _browser_build = packet.get_int(VarID::BrowserBuild);
    let user_type: Option<ClientType> = packet
        .get_int(VarID::UserType)
        .map(ClientType::from_i32)
        .unwrap();
    let username = packet.get_string(VarID::LoginUsername);
    let _password = packet.get_string(VarID::Password);
    let _email = packet.get_string(VarID::Email);
    let _privilege_id = packet.get_int(VarID::PrivilegeUserID);
    let _privilege_password = packet.get_string(VarID::PrivilegePassword);

    let mut response = AWPacket::new(PacketType::Login);
    let mut rc = ReasonCode::Success;

    match user_type {
        Some(user_type) => match user_type {
            ClientType::Bot => {
                todo!();
            }
            ClientType::UnspecifiedHuman => {
                client.info_mut().session_id = Some(client_manager.create_session_id());

                response.add_var(AWPacketVar::String(
                    VarID::CitizenName,
                    username.unwrap_or_else(|| "".to_string()),
                ));
                response.add_var(AWPacketVar::Int(
                    VarID::SessionID,
                    client.info().session_id.unwrap().into(),
                ));
            }
            _ => {
                rc = ReasonCode::NoSuchCitizen;
            }
        },
        None => {
            rc = ReasonCode::NoSuchCitizen;
        }
    }

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
    response.add_var(AWPacketVar::Data(
        VarID::UniverseLicense,
        license_generator.create_license_data(&client.info()),
    ));

    client.connection.send(response);
}
