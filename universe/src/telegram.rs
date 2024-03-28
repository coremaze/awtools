use aw_core::{AWPacket, PacketType};

use crate::{
    client::ClientInfo, database::TelegramDB, get_conn, player::Player,
    universe_connection::UniverseConnectionID, UniverseServer,
};

pub fn send_telegram_update_available(server: &UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn!(server, cid, "send_telegram_update_available");

    if let Some(ClientInfo::Player(Player::Citizen(citizen))) = &conn.client {
        let telegrams = server.database.telegram_get_undelivered(citizen.cit_id);
        if !telegrams.is_empty() {
            let packet = AWPacket::new(PacketType::TelegramNotify);
            conn.send(packet);
        }
    }
}
