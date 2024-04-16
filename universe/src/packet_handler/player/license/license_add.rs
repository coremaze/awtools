use crate::{
    database::LicenseDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

use super::{check_valid_world_name, license_from_packet};

pub fn license_add(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut p = AWPacket::new(PacketType::LicenseChangeResult);

    let conn = get_conn!(server, cid, "license_add");

    let Some(world_name) = packet.get_string(VarID::WorldName) else {
        p.add_int(
            VarID::ReasonCode,
            ReasonCode::NameContainsInvalidBlank as i32,
        );
        conn.send(p);
        return;
    };

    if !conn.has_admin_permissions() {
        log::trace!("Failed to add license due to lack of admin permissions");
        p.add_int(VarID::ReasonCode, ReasonCode::Unauthorized as i32);
        conn.send(p);
        return;
    }

    if world_name.contains(' ') || world_name.is_empty() {
        log::trace!("Failed to add license due to invalid name");
        p.add_int(VarID::ReasonCode, ReasonCode::NoSuchLicense as i32);
        conn.send(p);
        return;
    }

    let lic = match license_from_packet(packet) {
        Ok(x) => x,
        Err(why) => {
            log::info!("Couldn't get license from packet: {why}");
            return;
        }
    };

    match server.database.license_by_name(&lic.name) {
        DatabaseResult::Ok(Some(_)) => {
            p.add_int(VarID::ReasonCode, ReasonCode::WorldAlreadyExists.into());
            conn.send(p);
            return;
        }
        DatabaseResult::Ok(_) => {}
        DatabaseResult::DatabaseError => {
            p.add_int(VarID::ReasonCode, ReasonCode::DatabaseError.into());
            conn.send(p);
            return;
        }
    }

    if let Err(e) = check_valid_world_name(&lic.name) {
        p.add_int(VarID::ReasonCode, e as i32);
        conn.send(p);
        return;
    }

    if server.database.license_add(&lic).is_err() {
        p.add_int(VarID::ReasonCode, ReasonCode::UnableToInsertName as i32);
        conn.send(p);
        return;
    }

    p.add_int(VarID::ReasonCode, ReasonCode::Success as i32);
    conn.send(p);
}
