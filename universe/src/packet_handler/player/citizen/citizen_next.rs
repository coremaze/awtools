use crate::{
    database::CitizenDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;

use super::try_citizen_lookup;

struct CitizenNextParams {
    citizen_id: u32,
}

#[derive(Debug)]
enum CitizenNextParamsError {
    CitizenID,
}

impl TryFrom<&AWPacket> for CitizenNextParams {
    type Error = CitizenNextParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let citizen_id = value
            .get_uint(VarID::CitizenNumber)
            .ok_or(CitizenNextParamsError::CitizenID)?;

        Ok(Self { citizen_id })
    }
}

pub fn citizen_next(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let params = match CitizenNextParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete citizen next: {why:?}");
            return;
        }
    };

    let mut response = AWPacket::new(PacketType::CitizenInfo);
    let conn = get_conn!(server, cid, "citizen_next");

    let rc = try_citizen_lookup(conn, &mut response, || {
        server
            .database
            .citizen_by_number(params.citizen_id.saturating_add(1))
    });
    response.add_int(VarID::ReasonCode, rc.into());

    conn.send(response);
}
