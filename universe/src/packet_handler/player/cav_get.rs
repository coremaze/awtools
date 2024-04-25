use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{get_conn, universe_connection::UniverseConnectionID, UniverseServer};

pub fn get_cav(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "get_cav");

    let Some(_citizen_id) = packet.get_uint(VarID::CAVCitizen) else {
        return;
    };

    // CAV is not really implemented right now because I don't even know what they do in this version.
    // This packet is needed because otherwise it takes a long time to get PAV.
    let mut response = AWPacket::new(PacketType::CAVGet);
    response.add_uint(VarID::ReasonCode, ReasonCode::NoSuchCav.into());

    conn.send(response);
}
