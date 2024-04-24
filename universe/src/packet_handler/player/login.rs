use std::net::IpAddr;

use super::check_valid_name;
use crate::{
    client::ClientInfo,
    database::{citizen::CitizenQuery, CitizenDB, UniverseDatabase},
    ejection::is_connection_ejected,
    get_conn_mut,
    player::{Bot, Citizen, GenericPlayer, Player},
    tabs::{regenerate_contact_list_and_mutuals, regenerate_player_list, regenerate_world_list},
    telegram::send_telegram_update_available,
    timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID,
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;
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
            response.add_string(VarID::CitizenName, player.base_player().username.clone());
            response.add_int(VarID::SessionID, player.base_player().session_id.into());

            let conn = get_conn_mut!(server, cid, "login");
            log::info!(
                "{:?} is logging in as {}.",
                conn.addr().ip(),
                &player.base_player().username
            );

            new_clientinfo = Some(ClientInfo::Player(player));

            ReasonCode::Success
        }
        Err(rc) => rc,
    };

    add_license_data_to_packet(server, browser_build, &mut response);

    let conn = get_conn_mut!(server, cid, "login");
    conn.client = new_clientinfo;

    // This needs to be done after the connection's client info is created
    // because ejections can be based on the user's serial number,
    // which is parsed out of the login packet and stored in the client info
    match is_connection_ejected(&server.database, conn) {
        Some(true) => {
            conn.disconnect();
            log::info!(
                "Preventing {:?} from logging in because they are ejected",
                conn.addr().ip()
            );
            return;
        }
        Some(false) => {}
        None => log::warn!("Cannot determine whether client is ejected"),
    }

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

    match login_type {
        LoginType::WorldServer => {
            // A world server can't log in!
            Err(ReasonCode::NoSuchCitizen)
        }
        LoginType::UnspecifiedHuman => validate_human(server, cid, ip, packet, response),
        LoginType::Bot => validate_bot(server, ip, packet, response),
    }
}

fn validate_human(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    ip: IpAddr,
    packet: &AWPacket,
    response: &mut AWPacket,
) -> Result<Player, ReasonCode> {
    check_server_full(server)?;
    let username = packet
        .get_string(VarID::LoginUsername)
        .ok_or(ReasonCode::NoSuchCitizen)?;
    // let email = packet.get_string(VarID::Email);
    let privilege_id = packet.get_uint(VarID::PrivilegeUserID);
    let privilege_password = packet.get_string(VarID::PrivilegePassword);
    let browser_build = packet
        .get_int(VarID::BrowserBuild)
        .ok_or(ReasonCode::NoSuchCitizen)?;
    let serial = packet.get_uint(VarID::VolumeSerial);
    if let Some(serial) = serial {
        log::trace!("User logging in with serial {serial:X}");
    }

    // A user is a tourist if they have quotes around their name
    if username.starts_with('"') {
        check_tourist(server, &username)?;

        Ok(Player::Tourist(GenericPlayer {
            build: browser_build,
            session_id: server.connections.create_session_id(),
            privilege_id: None,
            username,
            nonce: None,
            world: None,
            ip,
            afk: false,
            serial,
            tabs: Default::default(),
        }))
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
            packet.get_data(17).as_ref(), // V6 only
            privilege_id,
            privilege_password.as_ref(),
        )?;

        // Add packet variables with citizen info
        response.add_uint(VarID::BetaUser, cit.beta);
        response.add_uint(VarID::TrialUser, cit.trial);
        response.add_uint(VarID::CitizenNumber, cit.id);
        response.add_uint(VarID::CitizenPrivacy, cit.privacy);
        response.add_uint(VarID::CAVEnabled, cit.cav_enabled);

        update_last_login_info(&server.database, &ip, cit.id);

        Ok(Player::Citizen(Citizen {
            cit_id: cit.id,
            base_player: GenericPlayer {
                build: browser_build,
                session_id: server.connections.create_session_id(),
                privilege_id,
                username: cit.name,
                nonce: None,
                world: None,
                ip,
                afk: false,
                serial,
                tabs: Default::default(),
            },
        }))
    }
}

fn validate_bot(
    server: &mut UniverseServer,
    ip: IpAddr,
    packet: &AWPacket,
    response: &mut AWPacket,
) -> Result<Player, ReasonCode> {
    let serial = packet.get_uint(VarID::VolumeSerial);

    // Build is typically much lower for SDK than for browsers
    let build = packet
        .get_int(VarID::BrowserBuild)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    // Bots need to specify the citizen ID of their owner
    let login_id = packet
        .get_uint(VarID::LoginID)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    if login_id == 0 {
        return Err(ReasonCode::NoSuchCitizen);
    }

    let _version = packet
        .get_int(VarID::BrowserVersion)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    // For bots, they need to send their name without brackets, but we respond
    // telling their name including the brackets
    let username = packet
        .get_string(VarID::LoginUsername)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    check_valid_name(&username, false)?;

    // A description of what the bot is supposed to do
    let application = packet
        .get_string(VarID::Application)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    // Bots log in using the privilege password of their owner
    let privilege_password = packet
        .get_string(VarID::PrivilegePassword)
        .ok_or(ReasonCode::InvalidPassword)?;

    // Normally also sends an AttributeCitizenChanges

    let cit_query = match server.database.citizen_by_number(login_id) {
        DatabaseResult::Ok(Some(cit_query)) => cit_query,
        DatabaseResult::Ok(None) => return Err(ReasonCode::NoSuchCitizen),
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    if privilege_password != cit_query.priv_pass {
        return Err(ReasonCode::InvalidPassword);
    }

    let bots_max = cit_query.bot_limit;
    let bots_owned = server
        .connections
        .iter()
        .map(|(_, conn)| conn)
        .filter(|conn| {
            // Count the number of bots
            if let Some(ClientInfo::Player(Player::Bot(bot))) = &conn.client {
                // Who are logged in with the same login id
                bot.owner_id == login_id
            } else {
                false
            }
        })
        .count();

    log::trace!("{login_id} currently has {bots_owned} bots, out of a limit of {bots_max}");

    // Administrator (ID 1) shouldn't be restricted by bot limit
    if bots_owned >= bots_max as usize && login_id != 1 {
        return Err(ReasonCode::BotLimitExceeded);
    }

    response.add_uint(VarID::BetaUser, cit_query.beta);
    response.add_uint(VarID::TrialUser, cit_query.trial);
    response.add_uint(VarID::CitizenNumber, cit_query.id);
    response.add_uint(VarID::CitizenPrivacy, cit_query.privacy);
    response.add_uint(VarID::CAVEnabled, cit_query.cav_enabled);
    response.add_string(VarID::PrivilegeUsername, cit_query.name);

    Ok(Player::Bot(Bot {
        owner_id: login_id,
        application,
        base_player: GenericPlayer {
            build,
            session_id: server.connections.create_session_id(),
            privilege_id: Some(login_id),
            username: format!("[{username}]"),
            nonce: None,
            world: None,
            ip,
            afk: false,
            serial,
            tabs: Default::default(),
        },
    }))
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
    let database_citizen = match server.database.citizen_by_name(username) {
        DatabaseResult::Ok(Some(cit)) => cit,
        DatabaseResult::Ok(None) => return Err(ReasonCode::NoSuchCitizen),
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

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
        let priv_citizen = match server.database.citizen_by_number(priv_id) {
            DatabaseResult::Ok(Some(cit)) => cit,
            DatabaseResult::Ok(None) => return Err(ReasonCode::NoSuchActingCitizen),
            DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
        };

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

fn add_license_data_to_packet(
    server: &UniverseServer,
    browser_build: Option<i32>,
    response: &mut AWPacket,
) {
    // Add license data (Specific to the IP/port binding that the client sees!)
    response.add_data(
        VarID::UniverseLicense,
        server
            .license_generator
            .create_license_data(browser_build.unwrap_or(0)),
    );
}

fn check_server_full(server: &UniverseServer) -> Result<(), ReasonCode> {
    let player_count = server
        .connections
        .iter()
        .filter(|(_id, conn)| conn.is_player() && !conn.is_bot())
        .count();

    if player_count >= usize::from(server.config.player_limit) {
        Err(ReasonCode::UniverseFull)
    } else {
        Ok(())
    }
}

fn update_last_login_info(database: &UniverseDatabase, address: &IpAddr, cit_id: u32) {
    let IpAddr::V4(ipv4) = address else {
        return;
    };

    let ip_u32 = u32::from_le_bytes(ipv4.octets());

    let mut cit_query = match database.citizen_by_number(cit_id) {
        DatabaseResult::Ok(Some(q)) => q,
        DatabaseResult::Ok(None) => {
            log::debug!("Can't get last login info because cit id {cit_id} does not exist.");
            return;
        }
        DatabaseResult::DatabaseError => {
            log::debug!("Can't get last login info due to database failure");
            return;
        }
    };

    cit_query.last_address = ip_u32;
    cit_query.last_login = unix_epoch_timestamp_u32();

    if let DatabaseResult::DatabaseError = database.citizen_change(&cit_query) {
        log::debug!("Can't update last login info due to database failure");
    }
}
