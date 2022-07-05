use crate::{
    client::{Client, ClientManager, ClientType, Entity, PlayerInfo},
    database::citizen::CitizenQuery,
    database::Database,
    license::LicenseGenerator,
};
use aw_core::*;
use num_traits::FromPrimitive;

/// Represents the credentials obtained during handling of the Login packet.
struct LoginCredentials {
    pub user_type: Option<ClientType>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub privilege_id: Option<u32>,
    pub privilege_password: Option<String>,
}

impl LoginCredentials {
    /// Parses login credentials from a packet.
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

/// Handle a client attempting to log in.
pub fn login(
    client: &Client,
    packet: &AWPacket,
    client_manager: &ClientManager,
    license_generator: &LicenseGenerator,
    database: &Database,
) {
    let _client_version = packet.get_int(VarID::BrowserVersion);
    let browser_build = packet.get_int(VarID::BrowserBuild);

    let credentials = LoginCredentials::from_packet(packet);

    let mut response = AWPacket::new(PacketType::Login);

    let rc = match validate_login(client, &credentials, client_manager, database) {
        // Successful login
        Ok(user) => {
            match (user, credentials.user_type) {
                // Promote to citizen
                (Some(citizen), Some(ClientType::UnspecifiedHuman)) => {
                    client.info_mut().client_type = Some(ClientType::Citizen);

                    let client_entity = Entity::Player(PlayerInfo {
                        build: browser_build.unwrap_or(0),
                        session_id: client_manager.create_session_id(),
                        citizen_id: Some(citizen.id),
                        username: citizen.name,
                    });

                    client.info_mut().entity = Some(client_entity);

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

                    let client_entity = Entity::Player(PlayerInfo {
                        build: browser_build.unwrap_or(0),
                        session_id: client_manager.create_session_id(),
                        citizen_id: None,
                        username: credentials.username.unwrap_or_default(),
                    });

                    client.info_mut().entity = Some(client_entity);
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

    // Inform the client of their displayed username and their new session ID
    if let Some(Entity::Player(info)) = &client.info_mut().entity {
        response.add_var(AWPacketVar::String(
            VarID::CitizenName,
            info.username.clone(),
        ));
        response.add_var(AWPacketVar::Int(VarID::SessionID, info.session_id as i32));
    }

    // Add license data (Specific to the IP/port binding that the client sees!)
    response.add_var(AWPacketVar::Data(
        VarID::UniverseLicense,
        license_generator.create_license_data(browser_build.unwrap_or(0)),
    ));

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
    client.connection.send(response);
}

/// Validates a client's login credentials.
/// This includes ensuring a valid username, the correct password(s) if applicable,
/// and the correct user type (world/bot/citizen/tourist).
/// Returns information about the citizen whose credentials matched (if not a tourist),
/// or returns a ReasonCode if login should fail.
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

/// Validate's human's login credentials. This applies to tourists and citizens
/// but not bots or worlds.
/// Returns information about the citizen whose credentials matched (if not a tourist),
/// or returns a ReasonCode if login should fail.
fn validate_human_login(
    client: &Client,
    credentials: &LoginCredentials,
    client_manager: &ClientManager,
    database: &Database,
) -> Result<Option<CitizenQuery>, ReasonCode> {
    let username = credentials
        .username
        .as_ref()
        .ok_or(ReasonCode::NoSuchCitizen)?;

    // A user is a tourist if they have quotes around their name
    if username.starts_with('"') {
        client_manager.check_tourist(username)?;
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

pub fn heartbeat(client: &Client) {
    log::info!("Received heartbeat from {}", client.addr.ip());
}
