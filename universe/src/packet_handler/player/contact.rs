use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::ClientInfo,
    database::{
        contact::{ContactOptions, ContactQuery},
        CitizenDB, ContactDB, Database, TelegramDB,
    },
    get_conn, get_conn_mut,
    player::Player,
    tabs::{
        regenerate_contact_list, regenerate_contact_list_and_mutuals, ContactListEntry,
        ContactState,
    },
    telegram,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;

pub fn contact_add(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut response = AWPacket::new(PacketType::ContactAdd);
    let conn = get_conn!(server, cid, "contact_add");

    let rc = match try_add_contact(conn, packet, &server.database) {
        Ok((cit_id, cont_id)) => {
            if !server.database.contact_blocked(cit_id, cont_id)
                && server
                    .database
                    .contact_friend_requests_allowed(cont_id, cit_id)
            {
                alert_friend_request(cit_id, cont_id, server);
            }
            response.add_uint(VarID::ContactListCitizenID, cont_id);
            // response.add_uint(
            //     VarID::ContactListOptions,
            //     server.database.contact_get_default(cit_id).bits(),
            // );

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::info!("Contact add: {rc:?}");
    response.add_int(VarID::ReasonCode, rc as i32);

    conn.send(response);
    regenerate_contact_list(server, cid);
}

fn alert_friend_request(from: u32, to: u32, server: &UniverseServer) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as u32;

    let citizen = match server.database.citizen_by_number(from) {
        Ok(x) => x,
        _ => return,
    };

    // Create a telegram to alert user of friend request
    let source_username = citizen.name;
    if server
        .database
        .telegram_add(to, from, now, &format!("\n\x01({from}){source_username}\n"))
        .is_err()
    {
        return;
    }

    // Alert recipient of new telegram
    if let Some(target_cid) = server.connections.get_by_citizen_id(to) {
        telegram::send_telegram_update_available(server, target_cid);
    }
}

fn try_add_contact(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &Database,
) -> Result<(u32, u32), ReasonCode> {
    // Must be a player logged in as a citizen
    let client = conn.client.as_ref().ok_or(ReasonCode::NotLoggedIn)?;
    let citizen = client.citizen().ok_or(ReasonCode::NotLoggedIn)?;

    let citizen_id = citizen.cit_id;

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

pub fn set_afk(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn_mut!(server, cid, "set_afk");

    let Some(client) = &mut conn.client else {
        return;
    };

    let Some(citizen) = client.citizen_mut() else {
        return;
    };

    let afk_status = match packet.get_uint(VarID::AFKStatus) {
        Some(x) => x,
        None => return,
    };

    let is_afk = afk_status != 0;
    citizen.player_info.afk = is_afk;
    log::debug!(
        "{:?} AFK: {:?}",
        citizen.player_info.username,
        citizen.player_info.afk
    );

    // Really only need to regenerate mutuals
    regenerate_contact_list_and_mutuals(server, cid);
}

pub fn contact_confirm(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "contact_confirm");

    let rc = match try_contact_confirm(conn, packet, &server.database) {
        Ok(_) => ReasonCode::Success,
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::ContactConfirm);
    response.add_int(VarID::ReasonCode, rc as i32);
    conn.send(response);
    regenerate_contact_list_and_mutuals(server, cid);
}

fn try_contact_confirm(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &Database,
) -> Result<(), ReasonCode> {
    // Must be a player logged in as a citizen
    let client = conn.client.as_ref().ok_or(ReasonCode::NotLoggedIn)?;
    let citizen = client.citizen().ok_or(ReasonCode::NotLoggedIn)?;

    let citizen_id = citizen.cit_id;

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

pub fn contact_list(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let Some(starting_id) = packet.get_uint(VarID::ContactListCitizenID) else {
        return;
    };
    let citizen_id = {
        let conn = get_conn_mut!(server, cid, "contact_list");
        let Some(ClientInfo::Player(Player::Citizen(citizen))) = &conn.client else {
            return;
        };
        citizen.cit_id
    };

    let contacts = server.database.contact_get_all(citizen_id);
    let mut entries = Vec::<ContactListEntry>::new();
    for contact in &contacts {
        entries.push(contact_entry(contact, server));
    }
    let conn = get_conn_mut!(server, cid, "contact_list");

    let ip = conn.addr().ip();

    let Some(player) = conn.player_info() else {
        return;
    };

    let name = player.username.clone();

    let current_list = player.tabs.contact_list.current_starting_from(starting_id);

    log::debug!(
        "Sending the CURRENT contact list starting from id {starting_id} to {} ({}) current: {:?}",
        ip,
        name,
        current_list
    );

    current_list.send_limited_list(conn);
}

pub fn contact_entry(contact: &ContactQuery, server: &UniverseServer) -> ContactListEntry {
    let mut username = "".to_string();
    let mut world: Option<String> = None;

    let contact_citizen = match server.database.citizen_by_number(contact.contact) {
        Ok(x) => x,
        Err(_) => {
            return ContactListEntry {
                username,
                world,
                state: ContactState::Hidden,
                citizen_id: contact.contact,
                options: ContactOptions::default(),
            }
        }
    };

    username = contact_citizen.name;

    let mut status = match server.connections.get_by_citizen_id(contact.contact) {
        Some(cid) => match server.connections.get_connection(cid) {
            Some(conn) => match &conn.client {
                Some(ClientInfo::Player(p)) => {
                    world = p.player_info().world.clone();
                    if p.player_info().afk {
                        ContactState::Afk
                    } else {
                        ContactState::Online
                    }
                }
                _ => {
                    log::error!(
                        "Connection received in contact_name_world_state is not a citizen."
                    );
                    ContactState::Offline
                }
            },
            None => {
                log::error!("Got an invalid CID in contact_name_world_state");
                ContactState::Offline
            }
        },
        None => ContactState::Offline,
    };

    if !server
        .database
        .contact_status_allowed(contact.contact, contact.citizen)
    {
        status = ContactState::Unknown;
    }

    ContactListEntry {
        username,
        world,
        state: status,
        citizen_id: contact.contact,
        options: contact.options,
    }
}
