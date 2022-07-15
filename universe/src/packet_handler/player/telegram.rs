use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::{Client, ClientManager, Entity},
    database::CitizenDB,
    database::Database,
    database::{telegram::TelegramQuery, ContactDB, TelegramDB},
};
use aw_core::*;

pub fn telegram_send(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    let rc = match try_send_telegram_from_packet(client, packet, database) {
        Ok(citizen_id) => {
            // Alert recipient of new telegram
            if let Some(target_client) = client_manager.get_client_by_citizen_id(citizen_id) {
                send_telegram_update_available(target_client, database);
            }

            ReasonCode::Success
        }
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::TelegramSend);
    response.add_int(VarID::ReasonCode, rc as i32);

    client.connection.send(response);
}

fn try_send_telegram_from_packet(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
) -> Result<u32, ReasonCode> {
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

    if !database.contact_telegrams_allowed(citizen_id, target_citizen.id)
        || !database.contact_telegrams_allowed(target_citizen.id, citizen_id)
    {
        return Err(ReasonCode::TelegramBlocked);
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs() as u32;

    database
        .telegram_add(target_citizen.id, citizen_id, now, &message)
        .map_err(|_| ReasonCode::UnableToSendTelegram)?;

    Ok(target_citizen.id)
}

pub fn send_telegram_update_available(client: &Client, database: &Database) {
    if let Some(Entity::Player(player)) = &client.info().entity {
        if let Some(citizen_id) = player.citizen_id {
            let telegrams = database.telegram_get_undelivered(citizen_id);
            if !telegrams.is_empty() {
                let packet = AWPacket::new(PacketType::TelegramNotify);
                client.connection.send(packet);
            }
        }
    }
}

pub fn telegram_get(client: &Client, packet: &AWPacket, database: &Database) {
    let mut response = AWPacket::new(PacketType::TelegramDeliver);

    let rc = match try_telegram_get(client, packet, database) {
        Ok((telegram, more_remain)) => {
            let from_name = match database.citizen_by_number(telegram.from) {
                Ok(cit) => cit.name,
                Err(_) => "<unknown>".to_string(),
            };
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Current time is before the unix epoch.")
                .as_secs() as u32;
            response.add_string(VarID::TelegramCitizenName, from_name);
            response.add_uint(VarID::TelegramAge, now.saturating_sub(telegram.timestamp));
            response.add_string(VarID::TelegramMessage, telegram.message);
            response.add_byte(VarID::TelegramsMoreRemain, more_remain as u8);

            ReasonCode::Success
        }
        Err(x) => x,
    };

    response.add_int(VarID::ReasonCode, rc as i32);

    client.connection.send(response);
}

pub fn try_telegram_get(
    client: &Client,
    _packet: &AWPacket,
    database: &Database,
) -> Result<(TelegramQuery, bool), ReasonCode> {
    let playerinfo = match &client.info().entity {
        Some(Entity::Player(x)) => x.clone(),
        _ => return Err(ReasonCode::UnableToGetTelegram),
    };

    let citizen_id = match playerinfo.citizen_id {
        Some(x) => x,
        None => return Err(ReasonCode::UnableToGetTelegram),
    };

    let telegrams = database.telegram_get_undelivered(citizen_id);

    let more_remain = telegrams.len() >= 2;

    if !telegrams.is_empty() {
        let telegram = telegrams[0].clone();
        database.telegram_mark_delivered(telegram.id).ok();
        Ok((telegrams[0].clone(), more_remain))
    } else {
        Err(ReasonCode::UnableToGetTelegram)
    }
}
