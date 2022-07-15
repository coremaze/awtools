use crate::{
    client::{Client, ClientType, Entity},
    database::citizen::CitizenQuery,
    database::CitizenDB,
    database::Database,
};
use aw_core::*;

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

    response.add_int(VarID::ReasonCode, rc as i32);

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

    response.add_int(VarID::ReasonCode, rc as i32);

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

    response.add_int(VarID::ReasonCode, rc as i32);

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

    response.add_int(VarID::ReasonCode, rc as i32);

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
    response.add_int(VarID::ReasonCode, rc as i32);

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

pub fn citizen_add(client: &Client, packet: &AWPacket, database: &Database) {
    let mut response = AWPacket::new(PacketType::CitizenChangeResult);
    let rc = match try_add_citizen(client, packet, database) {
        Ok(new_cit) => {
            response.add_uint(VarID::CitizenNumber, new_cit.id);
            response.add_string(VarID::CitizenName, new_cit.name);

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::trace!("Add citizen: {:?}", rc);
    response.add_int(VarID::ReasonCode, rc as i32);

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
