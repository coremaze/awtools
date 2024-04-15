use crate::{
    database::CitizenDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;

use super::try_citizen_lookup;

struct CitizenPrevParams {
    citizen_id: u32,
}

#[derive(Debug)]
enum CitizenPrevParamsError {
    CitizenID,
}

impl TryFrom<&AWPacket> for CitizenPrevParams {
    type Error = CitizenPrevParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let citizen_id = value
            .get_uint(VarID::CitizenNumber)
            .ok_or(CitizenPrevParamsError::CitizenID)?;

        Ok(Self { citizen_id })
    }
}

pub fn citizen_prev(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let params = match CitizenPrevParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete citizen prev: {why:?}");
            return;
        }
    };

    let mut response = AWPacket::new(PacketType::CitizenInfo);
    let conn = get_conn!(server, cid, "citizen_prev");

    let rc = try_citizen_lookup(conn, &mut response, || {
        server
            .database
            .citizen_by_number(params.citizen_id.saturating_sub(1))
    });
    response.add_int(VarID::ReasonCode, rc.into());

    conn.send(response);
}
