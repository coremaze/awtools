use crate::{
    client::{ClientManager, Entity},
    database::{citizen::CitizenQuery, Database},
    player::{PlayerInfo, PlayerState},
    universe_license::LicenseGenerator,
    Client, ClientType,
};
use aw_core::{AWPacket, AWPacketVar, PacketType, ReasonCode, VarID};
use num_traits::FromPrimitive;

use super::{send_telegram_update_available, update_contacts_of_user};

/// Represents the credentials obtained during handling of the Login packet.
struct LoginCredentials {
    pub user_type: Option<ClientType>,
    pub username: Option<String>,

    pub password: Option<String>,
    pub password_hash: Option<Vec<u8>>,

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

            #[cfg(feature = "protocol_v4")]
            password: packet.get_string(VarID::Password),
            #[cfg(feature = "protocol_v4")]
            password_hash: None,

            #[cfg(feature = "protocol_v6")]
            password_hash: packet.get_data(VarID::AttributeUserlist),
            #[cfg(feature = "protocol_v6")]
            password: None,

            email: packet.get_string(VarID::Email),
            privilege_id: packet.get_uint(VarID::PrivilegeUserID),
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

                    client.info_mut().entity = Some(Entity::new_citizen(
                        citizen.id,
                        credentials.privilege_id,
                        client_manager.create_session_id(),
                        browser_build.unwrap_or(0),
                        &citizen.name,
                        client.addr.ip(),
                    ));

                    // Update the user's friends to tell them this user is online
                    update_contacts_of_user(citizen.id, database, client_manager);

                    // Add packet variables with citizen info
                    response.add_uint(VarID::BetaUser, citizen.beta);
                    response.add_uint(VarID::TrialUser, citizen.trial);
                    response.add_uint(VarID::CitizenNumber, citizen.id);
                    response.add_uint(VarID::CitizenPrivacy, citizen.privacy);
                    response.add_uint(VarID::CAVEnabled, citizen.cav_enabled);

                    // TODO: update login time and last address
                }
                // Promote to tourist
                (None, Some(ClientType::UnspecifiedHuman)) => {
                    client.info_mut().client_type = Some(ClientType::Tourist);

                    client.info_mut().entity = Some(Entity::new_tourist(
                        client_manager.create_session_id(),
                        browser_build.unwrap_or(0),
                        &credentials.username.unwrap_or_default(),
                        client.addr.ip(),
                    ));
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
        response.add_string(VarID::CitizenName, info.username.clone());
        response.add_int(VarID::SessionID, info.session_id as i32);
    }

    // Add license data (Specific to the IP/port binding that the client sees!)
    response.add_data(
        VarID::UniverseLicense,
        license_generator.create_license_data(browser_build.unwrap_or(0)),
    );

    response.add_int(VarID::ReasonCode, rc as i32);
    client.connection.send(response);
    PlayerInfo::send_updates_to_all(&client_manager.get_player_infos(), client_manager);

    // Inform the client of new telegrams if they are available
    send_telegram_update_available(client, database);
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
            credentials.password.as_ref(),
            credentials.password_hash.as_ref(),
            credentials.privilege_id,
            &credentials.privilege_password,
        )?;
        Ok(Some(cit))
    }
}

pub fn heartbeat(client: &Client) {
    log::info!("Received heartbeat from {}", client.addr.ip());
}
