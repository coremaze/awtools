use crate::{
    database::CitizenDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;

use super::try_citizen_lookup;

struct CitizenLookupByNameParams {
    citizen_name: String,
}

#[derive(Debug)]
enum CitizenLookupByNameParamsError {
    CitizenName,
}

impl TryFrom<&AWPacket> for CitizenLookupByNameParams {
    type Error = CitizenLookupByNameParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let citizen_name = value
            .get_string(VarID::CitizenName)
            .ok_or(CitizenLookupByNameParamsError::CitizenName)?;

        Ok(Self { citizen_name })
    }
}

pub fn citizen_lookup_by_name(
    server: &UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let params = match CitizenLookupByNameParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete citizen lookup by name: {why:?}");
            return;
        }
    };

    let mut response = AWPacket::new(PacketType::CitizenInfo);

    let conn = get_conn!(server, cid, "citizen_lookup_by_name");

    let rc = try_citizen_lookup(conn, &mut response, || {
        server.database.citizen_by_name(&params.citizen_name)
    });

    response.add_int(VarID::ReasonCode, rc.into());

    conn.send(response);
}
