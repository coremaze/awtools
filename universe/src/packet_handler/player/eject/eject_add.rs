use std::net::IpAddr;

use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{
    database::EjectDB, get_conn, timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID, UniverseServer,
};

struct EjectAddParams {
    address: u32,
    expiration: u32,
    comment: String,
}

#[derive(Debug)]
enum EjectAddParamsError {
    Address,
    Expiration,
    Comment,
}

impl TryFrom<&AWPacket> for EjectAddParams {
    type Error = EjectAddParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let address = value
            .get_uint(VarID::EjectionAddress)
            .ok_or(EjectAddParamsError::Address)?;
        let expiration = value
            .get_uint(VarID::EjectionExpiration)
            .ok_or(EjectAddParamsError::Expiration)?;
        let comment = value
            .get_string(VarID::EjectionComment)
            .ok_or(EjectAddParamsError::Comment)?;

        Ok(Self {
            address,
            expiration,
            comment,
        })
    }
}

pub fn eject_add(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "eject_add");
    if !conn.has_admin_permissions() {
        log::trace!("eject_add failed because the client did not have permission");
        return;
    }

    let params = match EjectAddParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete eject add: {why:?}");
            return;
        }
    };

    let creation = unix_epoch_timestamp_u32();

    let rc = match server.database.ejection_set(
        params.address,
        params.expiration,
        creation,
        &params.comment,
    ) {
        aw_db::DatabaseResult::Ok(_) => {
            // Remove the ejected connection if it is present.
            if creation > params.expiration {
                for (_id, conn) in server.connections.iter_mut() {
                    let IpAddr::V4(ipv4) = conn.addr().ip() else {
                        continue;
                    };
                    let ip_u32 = u32::from_le_bytes(ipv4.octets());

                    if ip_u32 == params.address {
                        conn.disconnect();
                    }
                }
            }
            ReasonCode::Success
        }
        aw_db::DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    };

    let mut response = AWPacket::new(PacketType::EjectResult);
    response.add_uint(VarID::ReasonCode, rc.into());

    let conn = get_conn!(server, cid, "eject_add");
    conn.send(response);
}
