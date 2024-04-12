use aw_core::{AWPacket, PacketType};

use crate::{
    client::ClientInfo,
    database::{DatabaseResult, TelegramDB},
    get_conn,
    player::Player,
    universe_connection::UniverseConnectionID,
    UniverseServer,
};

pub fn send_telegram_update_available(server: &UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn!(server, cid, "send_telegram_update_available");

    let Some(ClientInfo::Player(Player::Citizen(citizen))) = &conn.client else {
        return;
    };

    match server.database.telegram_get_undelivered(citizen.cit_id) {
        DatabaseResult::Ok(telegrams) => {
            if !telegrams.is_empty() {
                let packet = AWPacket::new(PacketType::TelegramNotify);
                conn.send(packet);
            }
        }
        DatabaseResult::DatabaseError => {
            log::error!("Unable to complete send_telegram_update_available due to database error.")
        }
    }
}
