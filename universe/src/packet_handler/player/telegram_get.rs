use crate::{
    client::ClientInfo,
    database::{telegram::TelegramQuery, CitizenDB, TelegramDB, UniverseDatabase},
    get_conn,
    timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

pub fn telegram_get(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut response = AWPacket::new(PacketType::TelegramDeliver);
    let conn = get_conn!(server, cid, "telegram_get");

    let rc = match try_telegram_get(conn, packet, &server.database) {
        Ok((telegram, more_remain)) => match server.database.citizen_by_number(telegram.from) {
            DatabaseResult::Ok(cit) => {
                let from_name = match cit {
                    Some(cit) => cit.name,
                    None => "<unknown>".to_string(),
                };
                let now = unix_epoch_timestamp_u32();

                response.add_string(VarID::TelegramCitizenName, from_name);
                response.add_uint(VarID::TelegramAge, now.saturating_sub(telegram.timestamp));
                response.add_string(VarID::TelegramMessage, telegram.message);
                response.add_byte(VarID::TelegramsMoreRemain, more_remain as u8);

                ReasonCode::Success
            }
            DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
        },
        Err(x) => x,
    };

    response.add_int(VarID::ReasonCode, rc as i32);

    conn.send(response);
}

fn try_telegram_get(
    conn: &UniverseConnection,
    _packet: &AWPacket,
    database: &UniverseDatabase,
) -> Result<(TelegramQuery, bool), ReasonCode> {
    // Must be a player
    let Some(ClientInfo::Player(player)) = &conn.client else {
        return Err(ReasonCode::UnableToGetTelegram);
    };

    // Must be logged in as a citizen
    let Some(citizen_id) = player.citizen_id() else {
        return Err(ReasonCode::UnableToGetTelegram);
    };

    let telegrams = match database.telegram_get_undelivered(citizen_id) {
        DatabaseResult::Ok(telegrams) => telegrams,
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

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
