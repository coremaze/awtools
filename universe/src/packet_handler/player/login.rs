use crate::{
    client::{ClientInfo, Player, UniverseConnectionID},
    database::{citizen::CitizenQuery, CitizenDB},
    get_conn_mut,
    tabs::{regenerate_contact_list_and_mutuals, regenerate_player_list, regenerate_world_list},
    telegram::send_telegram_update_available,
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
pub enum LoginType {
    WorldServer = 1,
    UnspecifiedHuman = 2,
    Bot = 3,
    // Clients don't actually use these to log in, but they are able to be promoted to these
    // Citizen = 4,
    // Tourist = 5,
}

/// Handle a client attempting to log in.
pub fn login(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let _client_version = packet.get_int(VarID::BrowserVersion);
    let browser_build = packet.get_int(VarID::BrowserBuild);

    let mut response = AWPacket::new(PacketType::Login);

    let mut new_clientinfo: Option<ClientInfo> = None;

    let rc = match validate_login(server, cid, packet, &mut response) {
        Ok(player) => {
            // Inform the client of their displayed username and their new session ID
            response.add_string(VarID::CitizenName, player.player_info().username.clone());
            response.add_int(VarID::SessionID, player.player_info().session_id.into());

            let conn = get_conn_mut!(server, cid, "login");
            log::info!(
                "{:?} is logging in as {}.",
                conn.addr().ip(),
                &player.player_info().username
            );

            new_clientinfo = Some(ClientInfo::Player(player));

            ReasonCode::Success
        }
        Err(rc) => rc,
    };

    add_license_data_to_packet(server, browser_build, &mut response);

    let conn = get_conn_mut!(server, cid, "login");
    conn.client = new_clientinfo;

    response.add_int(VarID::ReasonCode, rc as i32);
    conn.send(response);

    for cid in server.connections.cids() {
        regenerate_player_list(server, cid)
    }
    regenerate_world_list(server, cid);
    regenerate_contact_list_and_mutuals(server, cid);
    // Inform the client of new telegrams if they are available
    send_telegram_update_available(server, cid);
    // update_contacts_of_user(server, cid);
}

/// Validates a client's login credentials.
/// This includes ensuring a valid username, the correct password(s) if applicable,
/// and the correct user type (world/bot/citizen/tourist).
/// Returns information about the citizen whose credentials matched (if not a tourist),
/// or returns a ReasonCode if login should fail.
fn validate_login(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
    response: &mut AWPacket,
) -> Result<Player, ReasonCode> {
    let Some(conn) = server.connections.get_connection(cid) else {
        log::error!("validate_login was given an invalid CID");
        return Err(ReasonCode::NoSuchCitizen);
    };

    let ip = conn.addr().ip();
    let login_type: LoginType = {
        let login_type_num = packet
            .get_int(VarID::UserType)
            .ok_or(ReasonCode::NoSuchCitizen)?;
        LoginType::from_i32(login_type_num).ok_or(ReasonCode::NoSuchCitizen)?
    };

    let username = packet
        .get_string(VarID::LoginUsername)
        .ok_or(ReasonCode::NoSuchCitizen)?;
    // let email = packet.get_string(VarID::Email);
    let privilege_id = packet.get_uint(VarID::PrivilegeUserID);
    let privilege_password = packet.get_string(VarID::PrivilegePassword);
    let browser_build = packet
        .get_int(VarID::BrowserBuild)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    match login_type {
        LoginType::WorldServer => {
            // A world server can't log in!
            Err(ReasonCode::NoSuchCitizen)
        }
        LoginType::UnspecifiedHuman => {
            // A user is a tourist if they have quotes around their name
            if username.starts_with('"') {
                check_tourist(server, &username)?;

                Ok(Player::new_tourist(
                    server.connections.create_session_id(),
                    browser_build,
                    &username,
                    ip,
                ))
            } else {
                #[cfg(feature = "protocol_v4")]
                let cit = check_citizen_v4(
                    server,
                    cid,
                    &username,
                    packet.get_string(VarID::Password).as_ref(), // V4 only
                    privilege_id,
                    privilege_password.as_ref(),
                )?;

                #[cfg(feature = "protocol_v6")]
                let cit = check_citizen_v6(
                    server,
                    cid,
                    &username,
                    packet.get_data(VarID::AttributeUserlist).as_ref(), // V6 only
                    privilege_id,
                    privilege_password.as_ref(),
                )?;

                // Add packet variables with citizen info
                response.add_uint(VarID::BetaUser, cit.beta);
                response.add_uint(VarID::TrialUser, cit.trial);
                response.add_uint(VarID::CitizenNumber, cit.id);
                response.add_uint(VarID::CitizenPrivacy, cit.privacy);
                response.add_uint(VarID::CAVEnabled, cit.cav_enabled);

                Ok(Player::new_citizen(
                    cit.id,
                    privilege_id,
                    server.connections.create_session_id(),
                    browser_build,
                    &cit.name,
                    ip,
                ))
            }
        }
        LoginType::Bot => {
            log::warn!("Bots are not yet implemented.");
            Err(ReasonCode::NoSuchCitizen)
        }
    }
}

#[cfg(feature = "protocol_v4")]
fn check_citizen_v4(
    server: &UniverseServer,
    cid: UniverseConnectionID,
    username: &str,
    password: Option<&String>,
    priv_id: Option<u32>,
    priv_pass: Option<&String>,
) -> Result<CitizenQuery, ReasonCode> {
    check_citizen_username(username)?;
    check_citizen_privilege(server, priv_id, priv_pass)?;

    // Get login citizen
    let database_citizen = server
        .database
        .citizen_by_name(username)
        .or(Err(ReasonCode::NoSuchCitizen))?;

    // Is login password correct?
    check_password(password, &database_citizen)?;

    check_citizen_enabled(&database_citizen)?;
    check_citizen_already_logged_in(server, &database_citizen, cid)?;

    Ok(database_citizen)
}

#[cfg(feature = "protocol_v6")]
fn check_citizen_v6(
    server: &UniverseServer,
    cid: UniverseConnectionID,
    username: &str,
    password_hash: Option<&Vec<u8>>,
    priv_id: Option<u32>,
    priv_pass: Option<&String>,
) -> Result<CitizenQuery, ReasonCode> {
    check_citizen_username(username)?;
    check_citizen_privilege(server, priv_id, priv_pass)?;

    // Get login citizen
    let database_citizen = server
        .database
        .citizen_by_name(username)
        .or(Err(ReasonCode::NoSuchCitizen))?;

    // Is login password correct?
    check_password_hash(&database_citizen, password_hash)?;

    check_citizen_enabled(&database_citizen)?;
    check_citizen_already_logged_in(server, &database_citizen, cid)?;

    Ok(database_citizen)
}

fn check_citizen_username(username: &str) -> Result<(), ReasonCode> {
    // Name and password must be present
    if username.is_empty() {
        return Err(ReasonCode::NoSuchCitizen);
    }

    // Name cannot be bot or tourist
    if username.starts_with('[') || username.starts_with('"') {
        return Err(ReasonCode::NoSuchCitizen);
    }

    Ok(())
}

fn check_citizen_privilege(
    server: &UniverseServer,
    priv_id: Option<u32>,
    priv_pass: Option<&String>,
) -> Result<(), ReasonCode> {
    // Checks if acquiring a privilege
    if let Some(priv_id) = priv_id.filter(|x| *x != 0) {
        // Get acting citizen
        let priv_citizen = server
            .database
            .citizen_by_number(priv_id)
            .map_err(|_| ReasonCode::NoSuchActingCitizen)?;

        // Is it enabled?
        if priv_citizen.enabled == 0 && priv_citizen.id != 1 {
            return Err(ReasonCode::NoSuchActingCitizen);
        }

        // Is the priv pass present and correct?
        let priv_pass = priv_pass.ok_or(ReasonCode::ActingPasswordInvalid)?;
        if *priv_pass != priv_citizen.priv_pass {
            return Err(ReasonCode::ActingPasswordInvalid);
        }
    }

    Ok(())
}

#[cfg(feature = "protocol_v4")]
fn check_password(
    password: Option<&String>,
    database_citizen: &CitizenQuery,
) -> Result<(), ReasonCode> {
    let password = password.ok_or(ReasonCode::InvalidPassword)?;
    if password.is_empty() {
        return Err(ReasonCode::InvalidPassword);
    }
    if database_citizen.password != *password {
        return Err(ReasonCode::InvalidPassword);
    }

    Ok(())
}

#[cfg(feature = "protocol_v6")]
fn check_password_hash(
    database_citizen: &CitizenQuery,
    password_hash: Option<&Vec<u8>>,
) -> Result<(), ReasonCode> {
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::io::Write;

    let mut correct_password_buf = Vec::<u8>::new();
    correct_password_buf
        .write_u32::<LittleEndian>(database_citizen.password.len() as u32)
        .map_err(|_| ReasonCode::InvalidPassword)?;
    correct_password_buf
        .write_all(
            &database_citizen
                .password
                .as_bytes()
                .iter()
                .rev()
                .map(|x| *x)
                .collect::<Vec<u8>>(),
        )
        .map_err(|_| ReasonCode::InvalidPassword)?;

    let hashed_correct_password = md5::compute(correct_password_buf.to_vec());

    let Some(password_hash) = password_hash else {
        return Err(ReasonCode::InvalidPassword);
    };

    if *password_hash != hashed_correct_password.to_vec() {
        return Err(ReasonCode::InvalidPassword);
    }

    Ok(())
}

fn check_citizen_enabled(database_citizen: &CitizenQuery) -> Result<(), ReasonCode> {
    // Is it enabled?
    if database_citizen.enabled == 0 {
        return Err(ReasonCode::CitizenDisabled);
    }
    Ok(())
}

fn check_citizen_already_logged_in(
    server: &UniverseServer,
    database_citizen: &CitizenQuery,
    this_cid: UniverseConnectionID,
) -> Result<(), ReasonCode> {
    // Is this citizen already logged in?
    for (other_cid, other_conn) in server.connections.iter() {
        let Some(other_client) = &other_conn.client else {
            continue;
        };
        if let Some(citizen) = other_client.citizen() {
            if citizen.cit_id == database_citizen.id {
                // Don't give an error if the client is already logged in as this user.
                if this_cid != *other_cid {
                    return Err(ReasonCode::IdentityAlreadyInUse);
                }
            }
        }
    }
    Ok(())
}

pub fn check_tourist(server: &UniverseServer, username: &str) -> Result<(), ReasonCode> {
    check_valid_name(username, true)?;

    for (_other_cid, other_conn) in server.connections.iter() {
        let Some(client_info) = &other_conn.client else {
            continue;
        };

        if let Some(tourist) = client_info.tourist() {
            if tourist.username == username {
                return Err(ReasonCode::NameAlreadyUsed);
            }
        }
    }

    Ok(())
}

fn check_valid_name(mut name: &str, is_tourist: bool) -> Result<(), ReasonCode> {
    if is_tourist {
        // Tourist names must start and end with quotes
        if !name.starts_with('"') || !name.ends_with('"') {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Strip quotes to continue check
        name = &name[1..name.len() - 1];
    }

    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if !name.chars().all(char::is_alphanumeric) {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}

fn add_license_data_to_packet(
    server: &UniverseServer,
    browser_build: Option<i32>,
    response: &mut AWPacket,
) {
    // Add license data (Specific to the IP/port binding that the client sees!)
    if let Some(browser_build) = browser_build {
        response.add_data(
            VarID::UniverseLicense,
            server.license_generator.create_license_data(browser_build),
        );
    }
}
