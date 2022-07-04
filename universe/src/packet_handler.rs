use crate::client;
use crate::database::citizen::{CitizenDB, CitizenQuery};
use crate::license::LicenseGenerator;
use crate::{
    attributes,
    client::{Client, ClientManager, ClientType},
    database::Database,
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

pub fn stream_key_response(client: &Client, packet: &AWPacket, database: &Database) {
    if let Some(encrypted_a4_key) = packet.get_data(VarID::EncryptionKey) {
        if let Ok(a4_key) = client.rsa.decrypt_private(&encrypted_a4_key) {
            client.connection.set_recv_key(&a4_key);
            attributes::send_attributes(client, database);
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

struct LoginCredentials {
    pub user_type: Option<ClientType>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub privilege_id: Option<u32>,
    pub privilege_password: Option<String>,
}

impl LoginCredentials {
    pub fn from_packet(packet: &AWPacket) -> Self {
        Self {
            user_type: packet
                .get_int(VarID::UserType)
                .map(ClientType::from_i32)
                .unwrap(),
            username: packet.get_string(VarID::LoginUsername),
            password: packet.get_string(VarID::Password),
            email: packet.get_string(VarID::Email),
            privilege_id: packet.get_int(VarID::PrivilegeUserID).map(|x| x as u32),
            privilege_password: packet.get_string(VarID::PrivilegePassword),
        }
    }
}

pub fn login(
    client: &Client,
    packet: &AWPacket,
    client_manager: &ClientManager,
    license_generator: &LicenseGenerator,
    database: &Database,
) {
    let _client_version = packet.get_int(VarID::BrowserVersion);
    let _browser_build = packet.get_int(VarID::BrowserBuild);

    let credentials = LoginCredentials::from_packet(packet);

    let mut response = AWPacket::new(PacketType::Login);

    let rc = match validate_login(client, &credentials, client_manager, database) {
        // Successful login
        Ok(user) => {
            match (user, credentials.user_type) {
                // Promote to citizen
                (Some(citizen), Some(ClientType::UnspecifiedHuman)) => {
                    client.info_mut().client_type = Some(ClientType::Citizen);
                    client.info_mut().username = Some(citizen.name);

                    // Add packet variables with citizen info
                    response.add_var(AWPacketVar::Int(VarID::BetaUser, citizen.beta as i32));
                    response.add_var(AWPacketVar::Int(VarID::TrialUser, citizen.trial as i32));
                    response.add_var(AWPacketVar::Int(VarID::CitizenNumber, citizen.id as i32));
                    response.add_var(AWPacketVar::Int(
                        VarID::CitizenPrivacy,
                        citizen.privacy as i32,
                    ));
                    response.add_var(AWPacketVar::Int(
                        VarID::CAVEnabled,
                        citizen.cav_enabled as i32,
                    ));

                    // TODO: update login time and last address
                }
                // Promote to tourist
                (None, Some(ClientType::UnspecifiedHuman)) => {
                    client.info_mut().client_type = Some(ClientType::Tourist);
                    client.info_mut().username = Some(credentials.username.unwrap_or_default());
                }
                (_, Some(ClientType::Bot)) => {
                    todo!();
                }
                _ => {
                    panic!("Got an OK login validation that wasn't a citizen, tourist, or bot. Should be impossible.");
                }
            }
            ReasonCode::Success
        }
        // Failed, either because of incorrect credentials or because the client is of the wrong type
        Err(reason) => reason,
    };

    // Assign user a session ID
    let new_session_id = client_manager.create_session_id();
    client.info_mut().session_id = Some(new_session_id);

    // Inform the client of their displayed username and their new session ID
    response.add_var(AWPacketVar::String(
        VarID::CitizenName,
        client.info().username.clone().unwrap_or_default(),
    ));
    response.add_var(AWPacketVar::Int(
        VarID::SessionID,
        client.info().session_id.unwrap_or_default() as i32,
    ));

    // Add license data (Specific to the IP/port binding that the client sees!)
    response.add_var(AWPacketVar::Data(
        VarID::UniverseLicense,
        license_generator.create_license_data(&client.info()),
    ));

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
    client.connection.send(response);
}

fn validate_login(
    client: &Client,
    credentials: &LoginCredentials,
    client_manager: &ClientManager,
    database: &Database,
) -> Result<Option<CitizenQuery>, ReasonCode> {
    match credentials.user_type {
        Some(ClientType::Bot) => todo!(),
        Some(ClientType::UnspecifiedHuman) => {
            validate_human_login(client, credentials, client_manager, database)
        }
        _ => Err(ReasonCode::NoSuchCitizen),
    }
}

fn validate_human_login(
    client: &Client,
    credentials: &LoginCredentials,
    client_manager: &ClientManager,
    database: &Database,
) -> Result<(Option<CitizenQuery>), ReasonCode> {
    let username = credentials
        .username
        .as_ref()
        .ok_or(ReasonCode::NoSuchCitizen)?;

    if username.starts_with("\"") {
        client_manager.check_tourist(&username)?;
        Ok(None)
    } else {
        let cit = client_manager.check_citizen(
            database,
            client,
            &credentials.username,
            &credentials.password,
            credentials.privilege_id,
            &credentials.privilege_password,
        )?;
        Ok(Some(cit))
    }
}
