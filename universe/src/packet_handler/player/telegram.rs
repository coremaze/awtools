use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    client::ClientInfo,
    database::{telegram::TelegramQuery, CitizenDB, ContactDB, Database, TelegramDB},
    get_conn,
    telegram::send_telegram_update_available,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;

pub fn telegram_send(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "telegram_send");
    let rc = match try_send_telegram_from_packet(conn, packet, &server.database) {
        Ok(citizen_id) => {
            // Alert recipient of new telegram
            if let Some(target_cid) = server.connections.get_by_citizen_id(citizen_id) {
                send_telegram_update_available(server, target_cid);
            }

            ReasonCode::Success
        }
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::TelegramSend);
    response.add_int(VarID::ReasonCode, rc as i32);

    conn.send(response);
}

fn try_send_telegram_from_packet(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &Database,
) -> Result<u32, ReasonCode> {
    // Must be a player
    let Some(ClientInfo::Player(player)) = &conn.client else {
        return Err(ReasonCode::NotLoggedIn);
    };

    // Must be logged in as a citizen
    let Some(citizen_id) = player.citizen_id() else {
        return Err(ReasonCode::NotLoggedIn);
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

pub fn telegram_get(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut response = AWPacket::new(PacketType::TelegramDeliver);
    let conn = get_conn!(server, cid, "telegram_get");

    let rc = match try_telegram_get(conn, packet, &server.database) {
        Ok((telegram, more_remain)) => {
            let from_name = match server.database.citizen_by_number(telegram.from) {
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

    conn.send(response);
}

pub fn try_telegram_get(
    conn: &UniverseConnection,
    _packet: &AWPacket,
    database: &Database,
) -> Result<(TelegramQuery, bool), ReasonCode> {
    // Must be a player
    let Some(ClientInfo::Player(player)) = &conn.client else {
        return Err(ReasonCode::UnableToGetTelegram);
    };

    // Must be logged in as a citizen
    let Some(citizen_id) = player.citizen_id() else {
        return Err(ReasonCode::UnableToGetTelegram);
    };

    let telegrams = database.telegram_get_undelivered(citizen_id);

    let telegram = telegrams.first();
    let more_remain = telegrams.len() >= 2;

    match telegram {
        Some(telegram) => {
            database.telegram_mark_delivered(telegram.id);
            Ok((telegram.clone(), more_remain))
        }
        None => Err(ReasonCode::UnableToGetTelegram),
    }
}
