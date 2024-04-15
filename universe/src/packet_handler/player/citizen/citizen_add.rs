use crate::{
    client::ClientInfo,
    database::{citizen::CitizenQuery, CitizenDB, UniverseDatabase},
    get_conn,
    player::Player,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

pub fn citizen_add(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let mut response = AWPacket::new(PacketType::CitizenChangeResult);

    let conn = get_conn!(server, cid, "citizen_add");

    let rc = match try_add_citizen(conn, packet, &server.database) {
        Ok(new_cit) => {
            response.add_uint(VarID::CitizenNumber, new_cit.id);
            response.add_string(VarID::CitizenName, new_cit.name);

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::trace!("Add citizen: {:?}", rc);
    response.add_int(VarID::ReasonCode, rc as i32);

    conn.send(response);
}

fn try_add_citizen(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &UniverseDatabase,
) -> Result<CitizenQuery, ReasonCode> {
    let id = packet
        .get_uint(VarID::CitizenNumber)
        .ok_or(ReasonCode::Unauthorized)?;
    let name = packet
        .get_string(VarID::CitizenName)
        .ok_or(ReasonCode::Unauthorized)?;
    let password = packet
        .get_string(VarID::CitizenPassword)
        .ok_or(ReasonCode::Unauthorized)?;
    let email = packet
        .get_string(VarID::CitizenEmail)
        .ok_or(ReasonCode::Unauthorized)?;
    let expiration = packet
        .get_uint(VarID::CitizenExpiration)
        .ok_or(ReasonCode::Unauthorized)?;
    let beta = packet
        .get_uint(VarID::BetaUser)
        .ok_or(ReasonCode::Unauthorized)?;
    let enabled = packet
        .get_uint(VarID::CitizenEnabled)
        .ok_or(ReasonCode::Unauthorized)?;
    let trial = packet
        .get_uint(VarID::TrialUser)
        .ok_or(ReasonCode::Unauthorized)?;
    let cav_enabled = packet
        .get_uint(VarID::CAVEnabled)
        .ok_or(ReasonCode::Unauthorized)?;

    let mut new_info = CitizenQuery {
        id,
        changed: 0,
        name,
        password,
        email,
        priv_pass: String::default(),
        comment: String::default(),
        url: String::default(),
        immigration: 0,
        expiration,
        last_login: 0,
        last_address: 0,
        total_time: 0,
        bot_limit: 0,
        beta,
        cav_enabled,
        cav_template: 0,
        enabled,
        privacy: 0,
        trial,
    };

    // Client needs to be an admin
    if !conn.has_admin_permissions() {
        return Err(ReasonCode::Unauthorized);
    }

    // Can't add citizen if another citizen already has the name
    match database.citizen_by_name(&new_info.name) {
        DatabaseResult::Ok(Some(_)) => return Err(ReasonCode::NameAlreadyUsed),
        DatabaseResult::Ok(None) => {}
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    }

    // Can't add citizen if someone already has the citzen number
    match database.citizen_by_number(new_info.id) {
        DatabaseResult::Ok(Some(_)) => return Err(ReasonCode::NumberAlreadyUsed),
        DatabaseResult::Ok(None) => {}
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    }

    // Can't add citizen if the id is too large
    if new_info.id > (i32::MAX as u32) {
        return Err(ReasonCode::UnableToInsertCitizen);
    }

    // Unimplemented: email filter

    if let Some(ClientInfo::Player(Player::Bot(_))) = conn.client {
        new_info.immigration = packet.get_uint(VarID::CitizenImmigration).unwrap_or(0);
        new_info.last_login = packet.get_uint(VarID::CitizenLastLogin).unwrap_or(0);
        new_info.total_time = packet.get_uint(VarID::CitizenTotalTime).unwrap_or(0);
    }

    let name = new_info.name.clone();

    let r = match new_info.id {
        0 => database.citizen_add_next(new_info),
        1.. => database.citizen_add(&new_info),
    };

    if r.is_err() {
        return Err(ReasonCode::UnableToInsertCitizen);
    }

    match database.citizen_by_name(&name) {
        DatabaseResult::Ok(Some(result)) => Ok(result),
        DatabaseResult::Ok(None) => Err(ReasonCode::UnableToInsertCitizen),
        DatabaseResult::DatabaseError => Err(ReasonCode::DatabaseError),
    }
}
