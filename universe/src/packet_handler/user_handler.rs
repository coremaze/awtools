use std::{
    net::IpAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    attributes,
    attributes::set_attribute,
    client::{Client, ClientManager, ClientType, Entity},
    database::citizen::CitizenQuery,
    database::{
        contact::{ContactOptions, ContactQuery},
        Database,
    },
    database::{license::LicenseQuery, ContactDB, TelegramDB},
    database::{CitizenDB, LicenseDB},
    license::LicenseGenerator,
    player::{PlayerInfo, PlayerState},
    world::World,
};
use aw_core::*;
use num_traits::FromPrimitive;
use rand::Rng;

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

                    let client_entity = Entity::Player(PlayerInfo {
                        build: browser_build.unwrap_or(0),
                        session_id: client_manager.create_session_id(),
                        citizen_id: Some(citizen.id),
                        privilege_id: credentials.privilege_id,
                        username: citizen.name,
                        nonce: None,
                        world: None,
                        ip: client.addr.ip(),
                        state: PlayerState::Online,
                    });

                    client.info_mut().entity = Some(client_entity);

                    // Add packet variables with citizen info
                    response.add_var(AWPacketVar::Uint(VarID::BetaUser, citizen.beta));
                    response.add_var(AWPacketVar::Uint(VarID::TrialUser, citizen.trial));
                    response.add_var(AWPacketVar::Uint(VarID::CitizenNumber, citizen.id));
                    response.add_var(AWPacketVar::Uint(VarID::CitizenPrivacy, citizen.privacy));
                    response.add_var(AWPacketVar::Uint(VarID::CAVEnabled, citizen.cav_enabled));

                    // TODO: update login time and last address
                }
                // Promote to tourist
                (None, Some(ClientType::UnspecifiedHuman)) => {
                    client.info_mut().client_type = Some(ClientType::Tourist);

                    let client_entity = Entity::Player(PlayerInfo {
                        build: browser_build.unwrap_or(0),
                        session_id: client_manager.create_session_id(),
                        citizen_id: None,
                        privilege_id: None,
                        username: credentials.username.unwrap_or_default(),
                        nonce: None,
                        world: None,
                        ip: client.addr.ip(),
                        state: PlayerState::Online,
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
    PlayerInfo::send_updates_to_all(&client_manager.get_player_infos(), client_manager);
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

fn ip_to_num(ip: IpAddr) -> u32 {
    let mut res: u32 = 0;
    if let std::net::IpAddr::V4(v4) = ip {
        for octet in v4.octets().iter().rev() {
            res <<= 8;
            res |= *octet as u32;
        }
    }
    res
}

pub fn user_list(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as i32;

    // I am not entirely sure what the purpose of this is, but it has some sort
    // of relation to 3 days. It sends our values back to us with this, so we
    // can use this to deny the client from spamming for updates, which causes
    // flickering of the user list with very large numbers of players.
    let time_val = packet.get_int(VarID::UserList3DayUnknown).unwrap_or(0);
    if now.saturating_sub(3) < time_val {
        return;
    }

    PlayerInfo::send_updates_to_one(&client_manager.get_player_infos(), client);
}

pub fn attribute_change(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    // Only admins should be able to change Universe attributes
    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to set attributes but is not an admin",
            client.addr.ip()
        );
        return;
    }

    for var in packet.get_vars().iter() {
        if let AWPacketVar::String(id, val) = var {
            log::info!("Client {} setting {:?} to {:?}", client.addr.ip(), id, val);
            set_attribute(*id, val, database).ok();
        }
    }

    for client in client_manager.clients() {
        attributes::send_attributes(client, database);
    }
}

pub fn citizen_next(client: &Client, packet: &AWPacket, database: &Database) {
    let mut rc = ReasonCode::Success;
    let mut response = AWPacket::new(PacketType::CitizenInfo);

    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to use CitizenNext but is not an admin",
            client.addr.ip()
        );
        rc = ReasonCode::Unauthorized;
    } else if let Some(Entity::Player(info)) = &client.info().entity {
        // TODO: next should be able to skip IDs
        let citizen_id = packet.get_uint(VarID::CitizenNumber).unwrap_or(0);
        match database.citizen_by_number(citizen_id.saturating_add(1)) {
            Ok(citizen) => {
                let same_citizen_id = Some(citizen.id) == info.citizen_id;
                let is_admin = client.has_admin_permissions();
                let vars = citizen_info_vars(&citizen, same_citizen_id, is_admin);
                for v in vars {
                    response.add_var(v);
                }
            }
            Err(_) => {
                rc = ReasonCode::NoSuchCitizen;
            }
        }
    }

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

pub fn citizen_prev(client: &Client, packet: &AWPacket, database: &Database) {
    let mut rc = ReasonCode::Success;
    let mut response = AWPacket::new(PacketType::CitizenInfo);

    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to use CitizenPrev but is not an admin",
            client.addr.ip()
        );
        rc = ReasonCode::Unauthorized;
    } else if let Some(Entity::Player(info)) = &client.info().entity {
        // TODO: prev should be able to skip IDs
        let citizen_id = packet.get_uint(VarID::CitizenNumber).unwrap_or(0);
        match database.citizen_by_number(citizen_id.saturating_sub(1)) {
            Ok(citizen) => {
                let same_citizen_id = Some(citizen.id) == info.citizen_id;
                let is_admin = client.has_admin_permissions();
                let vars = citizen_info_vars(&citizen, same_citizen_id, is_admin);
                for v in vars {
                    response.add_var(v);
                }
            }
            Err(_) => {
                rc = ReasonCode::NoSuchCitizen;
            }
        }
    }

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

pub fn citizen_lookup_by_name(client: &Client, packet: &AWPacket, database: &Database) {
    let mut rc = ReasonCode::Success;
    let mut response = AWPacket::new(PacketType::CitizenInfo);

    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to use CitizenLookupByName but is not an admin",
            client.addr.ip()
        );
        rc = ReasonCode::Unauthorized;
    } else if let Some(Entity::Player(info)) = &client.info().entity {
        match packet.get_string(VarID::CitizenName) {
            Some(citizen_name) => match database.citizen_by_name(&citizen_name) {
                Ok(citizen) => {
                    let same_citizen_id = Some(citizen.id) == info.citizen_id;
                    let is_admin = client.has_admin_permissions();
                    let vars = citizen_info_vars(&citizen, same_citizen_id, is_admin);
                    for v in vars {
                        response.add_var(v);
                    }
                }
                Err(_) => {
                    rc = ReasonCode::NoSuchCitizen;
                }
            },
            None => {
                rc = ReasonCode::NoSuchCitizen;
            }
        }
    }

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

pub fn citizen_lookup_by_number(client: &Client, packet: &AWPacket, database: &Database) {
    let mut rc = ReasonCode::Success;
    let mut response = AWPacket::new(PacketType::CitizenInfo);

    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to use CitizenLookupByNumber but is not an admin",
            client.addr.ip()
        );
        rc = ReasonCode::Unauthorized;
    } else if let Some(Entity::Player(info)) = &client.info().entity {
        match packet.get_uint(VarID::CitizenNumber) {
            Some(citizen_id) => match database.citizen_by_number(citizen_id) {
                Ok(citizen) => {
                    let same_citizen_id = Some(citizen.id) == info.citizen_id;
                    let is_admin = client.has_admin_permissions();
                    let vars = citizen_info_vars(&citizen, same_citizen_id, is_admin);
                    for v in vars {
                        response.add_var(v);
                    }
                }
                Err(_) => {
                    rc = ReasonCode::NoSuchCitizen;
                }
            },
            None => {
                rc = ReasonCode::NoSuchCitizen;
            }
        }
    }

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

pub fn citizen_change(client: &Client, packet: &AWPacket, database: &Database) {
    let changed_info = citizen_from_packet(packet);
    if changed_info.is_err() {
        log::trace!("Could not change citizen: {:?}", changed_info);
        return;
    }
    let changed_info = changed_info.unwrap();
    let mut rc = ReasonCode::Success;

    if let Some(Entity::Player(info)) = &client.info().entity {
        // Client needs to be the user in question or an admin
        if Some(changed_info.id) != info.citizen_id && !client.has_admin_permissions() {
            rc = ReasonCode::Unauthorized;
        } else {
            match database.citizen_by_number(changed_info.id) {
                Ok(original_info) => {
                    if let Err(x) = modify_citizen(
                        &original_info,
                        &changed_info,
                        database,
                        client.has_admin_permissions(),
                    ) {
                        rc = x;
                    }
                }
                Err(_) => {
                    rc = ReasonCode::NoSuchCitizen;
                }
            }
        }
    }

    let mut response = AWPacket::new(PacketType::CitizenChangeResult);
    log::trace!("Change citizen: {:?}", rc);
    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

fn modify_citizen(
    original: &CitizenQuery,
    changed: &CitizenQuery,
    database: &Database,
    admin: bool,
) -> Result<(), ReasonCode> {
    // Find any citizens with the same name as the new name
    if let Ok(matching_cit) = database.citizen_by_name(&changed.name) {
        // If someone already has the name, it needs to be the same user
        if matching_cit.id != original.id {
            return Err(ReasonCode::NameAlreadyUsed);
        }
    }

    let cit_query = CitizenQuery {
        id: original.id,
        changed: 0,
        name: changed.name.clone(),
        password: changed.password.clone(),
        email: changed.email.clone(),
        priv_pass: changed.priv_pass.clone(),
        comment: if admin {
            changed.comment.clone()
        } else {
            original.comment.clone()
        },
        url: changed.url.clone(),
        immigration: original.immigration,
        expiration: if admin {
            changed.expiration
        } else {
            original.expiration
        },
        last_login: original.last_login,
        last_address: original.last_address,
        total_time: original.total_time,
        bot_limit: if admin {
            changed.bot_limit
        } else {
            original.bot_limit
        },
        beta: if admin { changed.beta } else { original.beta },
        cav_enabled: if admin {
            changed.cav_enabled
        } else {
            original.cav_enabled
        },
        cav_template: changed.cav_template,
        enabled: if admin {
            changed.enabled
        } else {
            original.enabled
        },
        privacy: changed.privacy,
        trial: if admin { changed.trial } else { original.trial },
    };

    database
        .citizen_change(&cit_query)
        .map_err(|_| ReasonCode::UnableToChangeCitizen)?;

    Ok(())
}

fn citizen_info_vars(
    citizen: &CitizenQuery,
    self_vars: bool,
    admin_vars: bool,
) -> Vec<AWPacketVar> {
    let mut vars = vec![
        AWPacketVar::Uint(VarID::CitizenNumber, citizen.id),
        AWPacketVar::String(VarID::CitizenName, citizen.name.clone()),
        AWPacketVar::String(VarID::CitizenURL, citizen.url.clone()),
        AWPacketVar::Byte(VarID::TrialUser, citizen.trial as u8),
        AWPacketVar::Byte(VarID::CAVEnabled, citizen.cav_enabled as u8),
        AWPacketVar::Uint(
            VarID::CAVTemplate,
            if citizen.cav_enabled != 0 {
                citizen.cav_template
            } else {
                0
            },
        ),
    ];

    if self_vars || admin_vars {
        vars.extend(vec![
            AWPacketVar::Uint(VarID::CitizenImmigration, citizen.immigration),
            AWPacketVar::Uint(VarID::CitizenExpiration, citizen.expiration),
            AWPacketVar::Uint(VarID::CitizenLastLogin, citizen.last_login),
            AWPacketVar::Uint(VarID::CitizenTotalTime, citizen.total_time),
            AWPacketVar::Uint(VarID::CitizenBotLimit, citizen.bot_limit),
            AWPacketVar::Byte(VarID::BetaUser, citizen.beta as u8),
            AWPacketVar::Byte(VarID::CitizenEnabled, citizen.enabled as u8),
            AWPacketVar::Uint(VarID::CitizenPrivacy, citizen.privacy),
            AWPacketVar::String(VarID::CitizenPassword, citizen.password.clone()),
            AWPacketVar::String(VarID::CitizenEmail, citizen.email.clone()),
            AWPacketVar::String(VarID::CitizenPrivilegePassword, citizen.priv_pass.clone()),
            AWPacketVar::Uint(VarID::CitizenImmigration, citizen.immigration),
        ]);
    }

    if admin_vars {
        vars.extend(vec![
            AWPacketVar::String(VarID::CitizenComment, citizen.comment.clone()),
            AWPacketVar::Uint(VarID::IdentifyUserIP, citizen.last_address),
        ]);
    }

    vars
}

fn citizen_from_packet(packet: &AWPacket) -> Result<CitizenQuery, String> {
    let username = packet
        .get_string(VarID::CitizenName)
        .ok_or_else(|| "No citizen name".to_string())?;
    let citizen_id = packet
        .get_uint(VarID::CitizenNumber)
        .ok_or_else(|| "No citizen number".to_string())?;
    let email = packet
        .get_string(VarID::CitizenEmail)
        .ok_or_else(|| "No citizen email".to_string())?;
    let priv_pass = packet
        .get_string(VarID::CitizenPrivilegePassword)
        .ok_or_else(|| "No citizen privilege password".to_string())?;
    let expiration = packet
        .get_uint(VarID::CitizenExpiration)
        .ok_or_else(|| "No citizen expiration".to_string())?;
    let bot_limit = packet
        .get_uint(VarID::CitizenBotLimit)
        .ok_or_else(|| "No citizen bot limit".to_string())?;
    let beta = packet
        .get_uint(VarID::BetaUser)
        .ok_or_else(|| "No citizen beta user".to_string())?;
    let enabled = packet
        .get_uint(VarID::CitizenEnabled)
        .ok_or_else(|| "No citizen enabled".to_string())?;
    let comment = packet
        .get_string(VarID::CitizenComment)
        .ok_or_else(|| "No citizen comment".to_string())?;
    let password = packet
        .get_string(VarID::CitizenPassword)
        .ok_or_else(|| "No citizen password".to_string())?;
    let url = packet
        .get_string(VarID::CitizenURL)
        .ok_or_else(|| "No citizen url".to_string())?;
    let cav_template = packet
        .get_uint(VarID::CAVTemplate)
        .ok_or_else(|| "No citizen cav template".to_string())?;
    let cav_enabled = packet
        .get_uint(VarID::CAVEnabled)
        .ok_or_else(|| "No citizen cav enabled".to_string())?;
    let privacy = packet
        .get_uint(VarID::CitizenPrivacy)
        .ok_or_else(|| "No citizen privacy".to_string())?;
    let trial = packet
        .get_uint(VarID::TrialUser)
        .ok_or_else(|| "No citizen trial".to_string())?;

    Ok(CitizenQuery {
        id: citizen_id,
        changed: 0,
        name: username,
        password,
        email,
        priv_pass,
        comment,
        url,
        immigration: 0,
        expiration,
        last_login: 0,
        last_address: 0,
        total_time: 0,
        bot_limit,
        beta,
        cav_enabled,
        cav_template,
        enabled,
        privacy,
        trial,
    })
}

pub fn license_add(client: &Client, packet: &AWPacket, database: &Database) {
    let mut p = AWPacket::new(PacketType::LicenseChangeResult);

    let _player_info = match &client.info().entity {
        Some(Entity::Player(info)) => info,
        _ => return,
    };

    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    if !client.has_admin_permissions() {
        log::trace!("Failed to add license due to lack of admin permissions");
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::Unauthorized as i32,
        ));
        client.connection.send(p);
        return;
    }

    if world_name.contains(' ') || world_name.is_empty() {
        log::trace!("Failed to add license due to invalid name");
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::NoSuchLicense as i32,
        ));
        client.connection.send(p);
        return;
    }

    let lic = match license_from_packet(packet) {
        Ok(x) => x,
        Err(_) => return,
    };

    if database.license_by_name(&lic.name).is_ok() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::WorldAlreadyExists as i32,
        ));
        client.connection.send(p);
        return;
    }

    if let Err(e) = check_valid_world_name(&lic.name) {
        p.add_var(AWPacketVar::Int(VarID::ReasonCode, e as i32));
        client.connection.send(p);
        return;
    }

    if database.license_add(&lic).is_err() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::UnableToInsertName as i32,
        ));
        client.connection.send(p);
        return;
    }

    p.add_var(AWPacketVar::Int(
        VarID::ReasonCode,
        ReasonCode::Success as i32,
    ));
    client.connection.send(p);
}

enum WorldLicenseLookupMethod {
    Previous,
    Exact,
    Next,
}

pub fn license_by_name(client: &Client, packet: &AWPacket, database: &Database) {
    send_license_lookup(client, packet, database, WorldLicenseLookupMethod::Exact);
}

pub fn license_next(client: &Client, packet: &AWPacket, database: &Database) {
    send_license_lookup(client, packet, database, WorldLicenseLookupMethod::Next);
}

pub fn license_prev(client: &Client, packet: &AWPacket, database: &Database) {
    send_license_lookup(client, packet, database, WorldLicenseLookupMethod::Previous);
}

fn send_license_lookup(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    method: WorldLicenseLookupMethod,
) {
    let mut p = AWPacket::new(PacketType::LicenseResult);

    // Only admins should be able to query for world licenses
    if !client.has_admin_permissions() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::Unauthorized as i32,
        ));
        client.connection.send(p);
        return;
    }

    // World name to iterate from should be included
    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    // Get the previous/same/next world license starting from the included world name
    let license_result = match method {
        WorldLicenseLookupMethod::Previous => database.license_prev(&world_name),
        WorldLicenseLookupMethod::Exact => database.license_by_name(&world_name),
        WorldLicenseLookupMethod::Next => database.license_next(&world_name),
    };

    let rc = match license_result {
        Ok(lic) => {
            // Attach world license info to packet
            let vars = license_to_vars(&lic, client.has_admin_permissions());

            for v in vars {
                p.add_var(v);
            }

            ReasonCode::Success
        }
        Err(_) => {
            // No world license was found before/same/after the given name
            ReasonCode::NoSuchLicense
        }
    };

    p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(p);
}

pub fn license_change(client: &Client, packet: &AWPacket, database: &Database) {
    let mut p = AWPacket::new(PacketType::LicenseResult);

    // Only admins should be able change world licenses
    if !client.has_admin_permissions() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::Unauthorized as i32,
        ));
        client.connection.send(p);
        return;
    }

    // Altered license should be included
    let changed_lic = match license_from_packet(packet) {
        Ok(lic) => lic,
        Err(_) => return,
    };

    // Validate world name
    if let Err(rc) = check_valid_world_name(&changed_lic.name) {
        p.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
        client.connection.send(p);
        return;
    }

    // Get the license to be changed
    let original_lic = match database.license_by_name(&changed_lic.name) {
        Ok(lic) => lic,
        Err(_) => {
            p.add_var(AWPacketVar::Int(
                VarID::ReasonCode,
                ReasonCode::NoSuchLicense as i32,
            ));
            client.connection.send(p);
            return;
        }
    };

    // Change license
    let new_lic = LicenseQuery {
        id: original_lic.id,
        name: original_lic.name.clone(),
        password: changed_lic.password.clone(),
        email: changed_lic.email.clone(),
        comment: changed_lic.comment.clone(),
        creation: original_lic.creation,
        expiration: changed_lic.expiration,
        last_start: original_lic.last_start,
        last_address: original_lic.last_address,
        users: changed_lic.users,
        world_size: changed_lic.world_size,
        hidden: changed_lic.hidden,
        changed: 0,
        tourists: changed_lic.tourists,
        voip: changed_lic.voip,
        plugins: changed_lic.plugins,
    };
    if database.license_change(&new_lic).is_err() {
        p.add_var(AWPacketVar::Int(
            VarID::ReasonCode,
            ReasonCode::UnableToChangeLicense as i32,
        ));
        client.connection.send(p);
        return;
    }

    if let Ok(lic) = database.license_by_name(&changed_lic.name) {
        let vars = license_to_vars(&lic, client.has_admin_permissions());

        for v in vars {
            p.add_var(v);
        }
    }

    // TODO: Kill existing world if it is now invalid/expired
    p.add_var(AWPacketVar::Int(
        VarID::ReasonCode,
        ReasonCode::Success as i32,
    ));
    client.connection.send(p);
}

fn license_to_vars(lic: &LicenseQuery, admin: bool) -> Vec<AWPacketVar> {
    let mut result = vec![
        AWPacketVar::String(VarID::WorldStartWorldName, lic.name.clone()),
        AWPacketVar::Uint(VarID::WorldLicenseID, lic.id),
        AWPacketVar::Uint(VarID::WorldLicenseUsers, lic.users),
        AWPacketVar::Uint(VarID::WorldLicenseRange, lic.world_size),
    ];

    if admin {
        result.extend(vec![
            AWPacketVar::String(VarID::WorldLicensePassword, lic.password.clone()),
            AWPacketVar::String(VarID::WorldLicenseEmail, lic.email.clone()),
            AWPacketVar::String(VarID::WorldLicenseComment, lic.comment.clone()),
            AWPacketVar::Uint(VarID::WorldLicenseCreation, lic.creation),
            AWPacketVar::Uint(VarID::WorldLicenseExpiration, lic.expiration),
            AWPacketVar::Uint(VarID::WorldLicenseLastStart, lic.last_start),
            AWPacketVar::Uint(VarID::WorldLicenseLastAddress, lic.last_address),
            AWPacketVar::Uint(VarID::WorldLicenseTourists, lic.tourists),
            AWPacketVar::Uint(VarID::WorldLicenseHidden, lic.hidden),
            AWPacketVar::Uint(VarID::WorldLicenseVoip, lic.voip),
            AWPacketVar::Uint(VarID::WorldLicensePlugins, lic.plugins),
        ]);
    }

    result
}

fn check_valid_world_name(name: &str) -> Result<(), ReasonCode> {
    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    // Should be 16 in AW 5, but AW 4 has a limit of 8
    if name.len() > 8 {
        return Err(ReasonCode::NameTooLong);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if !name.chars().all(char::is_alphanumeric) {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}

fn license_from_packet(packet: &AWPacket) -> Result<LicenseQuery, String> {
    let name = packet
        .get_string(VarID::WorldStartWorldName)
        .ok_or_else(|| "No world name".to_string())?;
    let password = packet
        .get_string(VarID::WorldLicensePassword)
        .ok_or_else(|| "No world password".to_string())?;
    let email = packet
        .get_string(VarID::WorldLicenseEmail)
        .ok_or_else(|| "No license email".to_string())?;
    let comment = packet
        .get_string(VarID::WorldLicenseComment)
        .ok_or_else(|| "No license comment".to_string())?;
    let expiration = packet
        .get_uint(VarID::WorldLicenseExpiration)
        .ok_or_else(|| "No license expiration".to_string())?;
    let hidden = packet
        .get_uint(VarID::WorldLicenseHidden)
        .ok_or_else(|| "No license hidden".to_string())?;
    let tourists = packet
        .get_uint(VarID::WorldLicenseTourists)
        .ok_or_else(|| "No license tourists".to_string())?;
    let users = packet
        .get_uint(VarID::WorldLicenseUsers)
        .ok_or_else(|| "No license users".to_string())?;
    let world_size = packet
        .get_uint(VarID::WorldLicenseRange)
        .ok_or_else(|| "No license world size".to_string())?;
    let voip = packet
        .get_uint(VarID::WorldLicenseVoip)
        .ok_or_else(|| "No license voip".to_string())?;
    let plugins = packet
        .get_uint(VarID::WorldLicensePlugins)
        .ok_or_else(|| "No license plugins".to_string())?;

    Ok(LicenseQuery {
        id: 0,
        name,
        password,
        email,
        comment,
        expiration,
        last_start: 0,
        last_address: 0,
        users,
        world_size,
        hidden,
        changed: 0,
        tourists,
        voip,
        plugins,
        creation: 0,
    })
}

pub fn world_list(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    if let Some(Entity::Player(_)) = client.info().entity {
    } else {
        return;
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as i32;

    // Like with UserList, I am not sure what the purpose of this is,
    // but its function is similar
    let time_val = packet.get_int(VarID::WorldList3DayUnknown).unwrap_or(0);
    if now.saturating_sub(3) < time_val {
        return;
    }

    World::send_updates_to_one(&client_manager.get_world_infos(), client);
}

pub fn world_lookup(client: &Client, packet: &AWPacket, client_manager: &ClientManager) {
    let world_name = match packet.get_string(VarID::WorldStartWorldName) {
        Some(x) => x,
        None => return,
    };

    let mut p = AWPacket::new(PacketType::WorldLookup);

    p.add_var(AWPacketVar::String(
        VarID::WorldStartWorldName,
        world_name.clone(),
    ));

    match client_manager.get_world_by_name(&world_name) {
        Some(world) => {
            let mut client_info = client.info_mut();
            if let Some(Entity::Player(info)) = &mut client_info.entity {
                // Build nonce
                let mut rand_bytes = [0u8; 256];
                rand::thread_rng().fill(&mut rand_bytes);

                let mut nonce = [0u8; 255];
                nonce.copy_from_slice(&rand_bytes[0..255]);
                info.nonce = Some(nonce);

                p.add_var(AWPacketVar::Uint(VarID::WorldAddress, ip_to_num(world.ip)));
                p.add_var(AWPacketVar::Uint(VarID::WorldPort, world.port as u32));
                p.add_var(AWPacketVar::Uint(VarID::WorldLicenseUsers, world.max_users));
                p.add_var(AWPacketVar::Uint(
                    VarID::WorldLicenseRange,
                    world.world_size,
                ));
                p.add_var(AWPacketVar::Data(VarID::WorldUserNonce, nonce.to_vec()));

                p.add_var(AWPacketVar::Int(
                    VarID::ReasonCode,
                    ReasonCode::Success as i32,
                ));
            }
        }
        None => {
            p.add_var(AWPacketVar::Int(
                VarID::ReasonCode,
                ReasonCode::NoSuchWorld as i32,
            ));
        }
    }

    client.connection.send(p);
}

pub fn citizen_add(client: &Client, packet: &AWPacket, database: &Database) {
    let mut response = AWPacket::new(PacketType::CitizenChangeResult);
    let rc = match try_add_citizen(client, packet, database) {
        Ok(new_cit) => {
            response.add_var(AWPacketVar::Uint(VarID::CitizenNumber, new_cit.id));
            response.add_var(AWPacketVar::String(VarID::CitizenName, new_cit.name));

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::trace!("Add citizen: {:?}", rc);
    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

fn try_add_citizen(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
) -> Result<CitizenQuery, ReasonCode> {
    let id = packet
        .get_uint(VarID::CitizenNumber)
        .ok_or(ReasonCode::Unauthorized)?;
    let name = packet
        .get_string(VarID::CitizenName)
        .ok_or(ReasonCode::Unauthorized)?;
    let password = packet
        .get_string(VarID::CitizenPassword)
        .ok_or(ReasonCode::Unauthorized)?;
    let email = packet
        .get_string(VarID::CitizenEmail)
        .ok_or(ReasonCode::Unauthorized)?;
    let expiration = packet
        .get_uint(VarID::CitizenExpiration)
        .ok_or(ReasonCode::Unauthorized)?;
    let beta = packet
        .get_uint(VarID::BetaUser)
        .ok_or(ReasonCode::Unauthorized)?;
    let enabled = packet
        .get_uint(VarID::CitizenEnabled)
        .ok_or(ReasonCode::Unauthorized)?;
    let trial = packet
        .get_uint(VarID::TrialUser)
        .ok_or(ReasonCode::Unauthorized)?;
    let cav_enabled = packet
        .get_uint(VarID::CAVEnabled)
        .ok_or(ReasonCode::Unauthorized)?;

    let mut new_info = CitizenQuery {
        id,
        changed: 0,
        name,
        password,
        email,
        priv_pass: String::default(),
        comment: String::default(),
        url: String::default(),
        immigration: 0,
        expiration,
        last_login: 0,
        last_address: 0,
        total_time: 0,
        bot_limit: 0,
        beta,
        cav_enabled,
        cav_template: 0,
        enabled,
        privacy: 0,
        trial,
    };

    // Client needs to be an admin
    if !client.has_admin_permissions() {
        return Err(ReasonCode::Unauthorized);
    }

    // Can't add citizen if another citizen already has the name
    if database.citizen_by_name(&new_info.name).is_ok() {
        return Err(ReasonCode::NameAlreadyUsed);
    }

    // Cannot have ID 0 - TODO: get default next ID
    if new_info.id == 0 {
        return Err(ReasonCode::NumberAlreadyUsed);
    }

    // Can't add citizen if someone already has the citzen number
    if database.citizen_by_number(new_info.id).is_ok() {
        return Err(ReasonCode::NumberAlreadyUsed);
    }

    // Can't add citizen if the id is too large
    if new_info.id > (i32::MAX as u32) {
        return Err(ReasonCode::UnableToInsertCitizen);
    }

    // Unimplemented: email filter

    if client.info().client_type == Some(ClientType::Bot) {
        new_info.immigration = packet.get_uint(VarID::CitizenImmigration).unwrap_or(0);
        new_info.last_login = packet.get_uint(VarID::CitizenLastLogin).unwrap_or(0);
        new_info.total_time = packet.get_uint(VarID::CitizenTotalTime).unwrap_or(0);
    }

    database
        .citizen_add(&new_info)
        .map_err(|_| ReasonCode::UnableToInsertCitizen)?;

    let result = database
        .citizen_by_name(&new_info.name)
        .map_err(|_| ReasonCode::UnableToInsertCitizen)?;

    Ok(result)
}

pub fn contact_add(client: &Client, packet: &AWPacket, database: &Database) {
    let mut response = AWPacket::new(PacketType::ContactAdd);

    let rc = match try_add_contact(client, packet, database) {
        Ok((cit_id, cont_id)) => {
            response.add_var(AWPacketVar::Uint(VarID::ContactListCitizenID, cont_id));
            response.add_var(AWPacketVar::Uint(
                VarID::ContactListOptions,
                database.contact_get_default(cit_id).bits(),
            ));

            ReasonCode::Success
        }
        Err(x) => x,
    };

    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

fn try_add_contact(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
) -> Result<(u32, u32), ReasonCode> {
    // Must be a player
    let player_info = match &client.info().entity {
        Some(Entity::Player(x)) => x.clone(),
        _ => return Err(ReasonCode::NotLoggedIn),
    };

    // Must be logged in as a citizen
    let citizen_id = match player_info.citizen_id {
        Some(x) => x,
        None => return Err(ReasonCode::NotLoggedIn),
    };

    let contact_name = packet
        .get_string(VarID::ContactListName)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    let contact_options = packet
        .get_uint(VarID::ContactListOptions)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    let contact_citizen = database
        .citizen_by_name(&contact_name)
        .map_err(|_| ReasonCode::NoSuchCitizen)?;

    let contact_perms = database.contact_or_default(citizen_id, contact_citizen.id);

    if !contact_perms.can_add_friend() {
        return Err(ReasonCode::ContactAddBlocked);
    }

    database
        .contact_set(citizen_id, contact_citizen.id, contact_options)
        .map_err(|_| ReasonCode::UnableToSetContact)?;

    Ok((citizen_id, contact_citizen.id))
}

pub fn telegram_send(client: &Client, packet: &AWPacket, database: &Database) {
    let rc = match try_send_telegram(client, packet, database) {
        // TODO: alert recipeint of new telegram
        Ok(_) => ReasonCode::Success,
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::TelegramSend);
    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

fn try_send_telegram(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
) -> Result<(), ReasonCode> {
    // Must be a player
    let player_info = match &client.info().entity {
        Some(Entity::Player(x)) => x.clone(),
        _ => return Err(ReasonCode::NotLoggedIn),
    };

    // Must be logged in as a citizen
    let citizen_id = match player_info.citizen_id {
        Some(x) => x,
        None => return Err(ReasonCode::NotLoggedIn),
    };

    // TODO: aw_citizen_privacy

    let username_to = packet
        .get_string(VarID::TelegramTo)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    let message = packet
        .get_string(VarID::TelegramMessage)
        .ok_or(ReasonCode::UnableToSendTelegram)?;

    let target_citizen = database
        .citizen_by_name(&username_to)
        .map_err(|_| ReasonCode::NoSuchCitizen)?;

    let contact_info = database.contact_or_default(citizen_id, target_citizen.id);

    if !contact_info.is_telegram_allowed() {
        return Err(ReasonCode::TelegramBlocked);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as u32;

    database
        .telegram_add(target_citizen.id, citizen_id, now, &message)
        .map_err(|_| ReasonCode::UnableToSendTelegram)?;

    Ok(())
}
