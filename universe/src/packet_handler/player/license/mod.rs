mod license_add;
pub use license_add::license_add;

mod license_by_name;
pub use license_by_name::license_by_name;

mod license_next;
pub use license_next::license_next;

mod license_prev;
pub use license_prev::license_prev;

mod license_delete;
pub use license_delete::license_delete;

use crate::{
    database::{license::LicenseQuery, LicenseDB, UniverseDatabase},
    get_conn,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

enum WorldLicenseLookupMethod {
    Previous,
    Exact,
    Next,
}

fn send_license_lookup(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &UniverseDatabase,
    method: WorldLicenseLookupMethod,
) {
    let mut p = AWPacket::new(PacketType::LicenseResult);

    // Only admins should be able to query for world licenses
    if !conn.has_admin_permissions() {
        p.add_int(VarID::ReasonCode, ReasonCode::Unauthorized as i32);
        conn.send(p);
        return;
    }

    // World name to iterate from should be included
    let world_name = match packet.get_string(VarID::WorldName) {
        Some(x) => x,
        None => return,
    };

    // Get the previous/same/next world license starting from the included world name
    let license_result = match method {
        WorldLicenseLookupMethod::Previous => database.license_prev(&world_name),
        WorldLicenseLookupMethod::Exact => database.license_by_name(&world_name),
        WorldLicenseLookupMethod::Next => database.license_next(&world_name),
    };

    let rc = match license_result {
        DatabaseResult::Ok(Some(lic)) => {
            // Attach world license info to packet
            let vars = license_to_vars(&lic, conn.has_admin_permissions());
            for v in vars {
                p.add_var(v);
            }
            ReasonCode::Success
        }
        DatabaseResult::Ok(None) => {
            // No world license was found before/same/after the given name
            ReasonCode::NoSuchLicense
        }
        DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    };

    p.add_int(VarID::ReasonCode, rc as i32);

    conn.send(p);
}

pub fn license_change(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut p = AWPacket::new(PacketType::LicenseResult);
    let conn = get_conn!(server, cid, "license_change");

    // Only admins should be able change world licenses
    if !conn.has_admin_permissions() {
        p.add_int(VarID::ReasonCode, ReasonCode::Unauthorized as i32);
        conn.send(p);
        return;
    }

    // Altered license should be included
    let changed_lic = match license_from_packet(packet) {
        Ok(lic) => lic,
        Err(_) => return,
    };

    // Validate world name
    if let Err(rc) = check_valid_world_name(&changed_lic.name) {
        p.add_int(VarID::ReasonCode, rc as i32);
        conn.send(p);
        return;
    }

    // Get the license to be changed
    let original_lic = match server.database.license_by_name(&changed_lic.name) {
        DatabaseResult::Ok(Some(lic)) => lic,
        DatabaseResult::Ok(None) => {
            p.add_int(VarID::ReasonCode, ReasonCode::NoSuchLicense.into());
            conn.send(p);
            return;
        }
        DatabaseResult::DatabaseError => {
            p.add_int(VarID::ReasonCode, ReasonCode::DatabaseError.into());
            conn.send(p);
            return;
        }
    };

    // Change license
    let new_lic = LicenseQuery {
        id: original_lic.id,
        name: original_lic.name.clone(),
        password: changed_lic.password.clone(),
        email: changed_lic.email.clone(),
        comment: changed_lic.comment.clone(),
        creation: original_lic.creation,
        expiration: changed_lic.expiration,
        last_start: original_lic.last_start,
        last_address: original_lic.last_address,
        users: changed_lic.users,
        world_size: changed_lic.world_size,
        hidden: changed_lic.hidden,
        changed: 0,
        tourists: changed_lic.tourists,
        voip: changed_lic.voip,
        plugins: changed_lic.plugins,
    };
    if server.database.license_change(&new_lic).is_err() {
        p.add_int(VarID::ReasonCode, ReasonCode::UnableToChangeLicense as i32);
        conn.send(p);
        return;
    }

    match server.database.license_by_name(&changed_lic.name) {
        DatabaseResult::Ok(Some(lic)) => {
            let vars = license_to_vars(&lic, conn.has_admin_permissions());
            for v in vars {
                p.add_var(v);
            }
        }
        DatabaseResult::Ok(None) => {}
        DatabaseResult::DatabaseError => {
            p.add_int(VarID::ReasonCode, ReasonCode::DatabaseError.into());
            conn.send(p);
            return;
        }
    }

    // TODO: Kill existing world if it is now invalid/expired
    p.add_int(VarID::ReasonCode, ReasonCode::Success.into());
    conn.send(p);
}

fn license_to_vars(lic: &LicenseQuery, admin: bool) -> Vec<AWPacketVar> {
    let mut result = vec![
        AWPacketVar::string(VarID::WorldName, lic.name.clone()),
        AWPacketVar::uint(VarID::WorldLicenseID, lic.id),
        AWPacketVar::uint(VarID::WorldLicenseUsers, lic.users),
        AWPacketVar::uint(VarID::WorldLicenseRange, lic.world_size),
    ];

    if admin {
        result.extend(vec![
            AWPacketVar::string(VarID::WorldLicensePassword, lic.password.clone()),
            AWPacketVar::string(VarID::WorldLicenseEmail, lic.email.clone()),
            AWPacketVar::string(VarID::WorldLicenseComment, lic.comment.clone()),
            AWPacketVar::uint(VarID::WorldLicenseCreation, lic.creation),
            AWPacketVar::uint(VarID::WorldLicenseExpiration, lic.expiration),
            AWPacketVar::uint(VarID::WorldLicenseLastStart, lic.last_start),
            AWPacketVar::uint(VarID::WorldLicenseLastAddress, lic.last_address),
            AWPacketVar::uint(VarID::WorldLicenseTourists, lic.tourists),
            AWPacketVar::uint(VarID::WorldLicenseHidden, lic.hidden),
            AWPacketVar::uint(VarID::WorldLicenseVoip, lic.voip),
            AWPacketVar::uint(VarID::WorldLicensePlugins, lic.plugins),
        ]);
    }

    result
}

fn check_valid_world_name(name: &str) -> Result<(), ReasonCode> {
    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    // Should be 16 in AW 5, but AW 4 has a limit of 8
    if name.len() > 8 {
        return Err(ReasonCode::NameTooLong);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if !name.chars().all(char::is_alphanumeric) {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}

fn license_from_packet(packet: &AWPacket) -> Result<LicenseQuery, String> {
    let name = packet
        .get_string(VarID::WorldName)
        .ok_or_else(|| "No world name".to_string())?;
    let password = packet
        .get_string(VarID::WorldLicensePassword)
        .ok_or_else(|| "No world password".to_string())?;
    let email = packet
        .get_string(VarID::WorldLicenseEmail)
        .ok_or_else(|| "No license email".to_string())?;
    let comment = packet
        .get_string(VarID::WorldLicenseComment)
        .ok_or_else(|| "No license comment".to_string())?;
    let expiration = packet
        .get_uint(VarID::WorldLicenseExpiration)
        .ok_or_else(|| "No license expiration".to_string())?;
    let hidden = packet
        .get_uint(VarID::WorldLicenseHidden)
        .ok_or_else(|| "No license hidden".to_string())?;
    let tourists = packet
        .get_uint(VarID::WorldLicenseTourists)
        .ok_or_else(|| "No license tourists".to_string())?;
    let users = packet
        .get_uint(VarID::WorldLicenseUsers)
        .ok_or_else(|| "No license users".to_string())?;
    let world_size = packet
        .get_uint(VarID::WorldLicenseRange)
        .ok_or_else(|| "No license world size".to_string())?;
    let voip = packet
        .get_uint(VarID::WorldLicenseVoip)
        .ok_or_else(|| "No license voip".to_string())?;
    let plugins = packet
        .get_uint(VarID::WorldLicensePlugins)
        .ok_or_else(|| "No license plugins".to_string())?;

    Ok(LicenseQuery {
        id: 0,
        name,
        password,
        email,
        comment,
        expiration,
        last_start: 0,
        last_address: 0,
        users,
        world_size,
        hidden,
        changed: 0,
        tourists,
        voip,
        plugins,
        creation: 0,
    })
}
