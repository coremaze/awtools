use crate::{
    database::CitizenDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;

use super::try_citizen_lookup;

struct CitizenLookupByNumberParams {
    citizen_number: u32,
}

#[derive(Debug)]
enum CitizenLookupByNumberParamsError {
    CitizenNumber,
}

impl TryFrom<&AWPacket> for CitizenLookupByNumberParams {
    type Error = CitizenLookupByNumberParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let citizen_number = value
            .get_uint(VarID::CitizenNumber)
            .ok_or(CitizenLookupByNumberParamsError::CitizenNumber)?;

        Ok(Self { citizen_number })
    }
}

pub fn citizen_lookup_by_number(
    server: &UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let params = match CitizenLookupByNumberParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete citizen lookup by number: {why:?}");
            return;
        }
    };

    let mut response = AWPacket::new(PacketType::CitizenInfo);

    let conn = get_conn!(server, cid, "citizen_lookup_by_number");

    let rc = try_citizen_lookup(conn, &mut response, || {
        server.database.citizen_by_number(params.citizen_number)
    });

    response.add_int(VarID::ReasonCode, rc.into());

    conn.send(response);
}
