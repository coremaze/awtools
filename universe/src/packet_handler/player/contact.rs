use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::{Client, ClientManager, Entity},
    database::contact::ContactQuery,
    database::CitizenDB,
    database::{contact::ContactOptions, Database},
    database::{ContactDB, TelegramDB},
    player::{PlayerInfo, PlayerState},
};
use aw_core::*;

use super::telegram;

#[derive(Debug)]
enum ContactState {
    Offline = 0,
    Online = 1,
    Nonexistent = 2,
    Afk = 3,
    Unknown = 4,
    Removed = 5,
    Default = 6,
}

pub fn contact_add(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    let mut response = AWPacket::new(PacketType::ContactAdd);

    let rc = match try_add_contact(client, packet, database) {
        Ok((cit_id, cont_id)) => {
            if !database.contact_blocked(cit_id, cont_id)
                && database.contact_friend_requests_allowed(cont_id, cit_id)
            {
                alert_friend_request(cit_id, cont_id, database, client_manager);
            }
            response.add_var(AWPacketVar::Uint(VarID::ContactListCitizenID, cont_id));
            // response.add_var(AWPacketVar::Uint(
            //     VarID::ContactListOptions,
            //     database.contact_get_default(cit_id).bits(),
            // ));

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::info!("Contact add: {rc:?}");
    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));

    client.connection.send(response);
}

fn alert_friend_request(from: u32, to: u32, database: &Database, client_manager: &ClientManager) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as u32;

    let citizen = match database.citizen_by_number(from) {
        Ok(x) => x,
        _ => return,
    };

    // Create a telegram to alert user of friend request
    let source_username = citizen.name;
    if database
        .telegram_add(to, from, now, &format!("\n\x01({from}){source_username}\n"))
        .is_err()
    {
        return;
    }

    // Alert recipient of new telegram
    if let Some(target_client) = client_manager.get_client_by_citizen_id(to) {
        telegram::send_telegram_update_available(target_client, database);
    }
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

    let mut options = ContactOptions::from_bits_truncate(contact_options);
    if database.contact_blocked(citizen_id, contact_citizen.id)
        && !options.contains(ContactOptions::ALL_BLOCKED)
    {
        return Err(ReasonCode::ContactAddBlocked);
    }

    // Stop people from adding each other when they are already friends
    if database
        .contact_get(citizen_id, contact_citizen.id)
        .is_err()
        || database
            .contact_get(contact_citizen.id, citizen_id)
            .is_err()
    {
        options.remove(ContactOptions::FRIEND_REQUEST_ALLOWED);
        options.insert(ContactOptions::FRIEND_REQUEST_BLOCKED);

        database
            .contact_set(citizen_id, contact_citizen.id, options.bits())
            .map_err(|_| ReasonCode::UnableToSetContact)?;
    }

    Ok((citizen_id, contact_citizen.id))
}

pub fn set_afk(client: &Client, packet: &AWPacket) {
    if let Some(Entity::Player(player)) = &mut client.info_mut().entity {
        if player.citizen_id.is_none() {
            return;
        }

        let afk_status = match packet.get_uint(VarID::AFKStatus) {
            Some(x) => x,
            None => return,
        };

        let is_afk = afk_status != 0;
        player.afk = is_afk;
        log::info!("{:?} AFK: {:?}", player.username, player.afk);
    }
}

pub fn contact_confirm(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    _client_manager: &ClientManager,
) {
    let rc = match try_contact_confirm(client, packet, database) {
        Ok(_) => ReasonCode::Success,
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::ContactConfirm);
    response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
    client.connection.send(response);
}

fn try_contact_confirm(
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

    let contact_id = packet
        .get_uint(VarID::ContactListCitizenID)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    if packet.get_int(VarID::ContactListOptions).unwrap_or(-1) == -1 {
        // Contact request denied
        return Ok(());
    }

    let contact_options = packet
        .get_uint(VarID::ContactListOptions)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    let target_options = match database.contact_get(contact_id, citizen_id) {
        Ok(x) => x.options,
        // Fail if the target has no contact for this citizen (i.e. this contact was not requested)
        _ => return Err(ReasonCode::UnableToSetContact),
    };

    if !target_options.is_friend_request_allowed() {
        return Err(ReasonCode::UnableToSetContact);
    }

    // Fail if could not set the contacts
    if database.contact_set(citizen_id, contact_id, 0).is_err()
        || database.contact_set(contact_id, citizen_id, 0).is_err()
    {
        return Err(ReasonCode::UnableToSetContact);
    }

    log::info!(
        "Accepted contact {:?}",
        ContactOptions::from_bits_truncate(contact_options)
    );

    Ok(())
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

pub fn contact_list(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    let groups = match try_contact_list(client, packet, database, client_manager) {
        Ok(groups) => groups,
        Err(rc) => {
            let mut response = AWPacket::new(PacketType::ContactList);
            response.add_var(AWPacketVar::Int(VarID::ReasonCode, rc as i32));
            client.connection.send(response);
            return;
        }
    };

    log::info!("Sending contact list: {groups:?}");
    for group in groups {
        client.connection.send_group(group);
    }
}

fn try_contact_list(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) -> Result<Vec<AWPacketGroup>, ReasonCode> {
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

    let contacts = database.contact_get_all(citizen_id);

    let groups = get_contact_list_groups(&contacts, database, client_manager);

    Ok(groups)
}

fn get_contact_list_groups(
    contacts: &[ContactQuery],
    database: &Database,
    client_manager: &ClientManager,
) -> Vec<AWPacketGroup> {
    let mut groups = Vec::<AWPacketGroup>::new();
    let mut group = AWPacketGroup::new();

    for contact in contacts {
        let (username, world, state) = contact_name_world_state(contact, database, client_manager);

        let mut response = AWPacket::new(PacketType::ContactList);
        response.add_var(AWPacketVar::String(VarID::ContactListName, username));
        response.add_var(AWPacketVar::String(VarID::ContactListWorld, world));
        response.add_var(AWPacketVar::Int(VarID::ContactListStatus, state as i32));
        response.add_var(AWPacketVar::Uint(
            VarID::ContactListCitizenID,
            contact.contact,
        ));
        response.add_var(AWPacketVar::Byte(VarID::ContactListMore, 1));
        response.add_var(AWPacketVar::Uint(
            VarID::ContactListOptions,
            contact.options.bits(),
        ));

        if let Err(p) = group.push(response) {
            groups.push(group);
            group = AWPacketGroup::new();
            group.push(p).ok();
        }
    }

    let mut response = AWPacket::new(PacketType::ContactList);
    response.add_var(AWPacketVar::Uint(VarID::ContactListCitizenID, 0));
    response.add_var(AWPacketVar::Byte(VarID::ContactListMore, 0));

    if let Err(p) = group.push(response) {
        groups.push(group);
        group = AWPacketGroup::new();
        group.push(p).ok();
    }

    groups.push(group);

    groups
}

pub fn update_contacts_of_user(
    citizen_id: u32,
    database: &Database,
    client_manager: &ClientManager,
) {
    for client in client_manager.clients() {
        if let Some(Entity::Player(player)) = &client.info().entity {
            if let Some(client_citizen_id) = player.citizen_id {
                let contact = match database.contact_get(client_citizen_id, citizen_id) {
                    Ok(contact) => contact,
                    Err(_) => continue,
                };
                let groups = get_contact_list_groups(&[contact], database, client_manager);
                for group in groups {
                    client.connection.send_group(group);
                }
            }
        }
    }
}

fn contact_name_world_state(
    contact: &ContactQuery,
    database: &Database,
    client_manager: &ClientManager,
) -> (String, String, ContactState) {
    let mut username = "".to_string();
    let mut world = "".to_string();

    let contact_citizen = match database.citizen_by_number(contact.contact) {
        Ok(x) => x,
        Err(_) => return (username, world, ContactState::Nonexistent),
    };

    username = contact_citizen.name;

    let mut status = match client_manager.get_client_by_citizen_id(contact.contact) {
        Some(client) => match &client.info().entity {
            Some(Entity::Player(player)) => match player.state {
                PlayerState::Offline => ContactState::Offline,
                PlayerState::Online => {
                    if let Some(player_world) = &player.world {
                        world = player_world.clone();
                    }
                    match player.afk {
                        true => ContactState::Afk,
                        false => ContactState::Online,
                    }
                }
            },
            _ => ContactState::Unknown,
        },
        None => ContactState::Offline,
    };

    if !database.contact_status_allowed(contact.contact, contact.citizen) {
        status = ContactState::Unknown;
    }

    (username, world, status)
}
