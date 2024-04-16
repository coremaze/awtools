use super::check_valid_name;
use crate::{
    database::{citizen::CitizenQuery, CitizenDB},
    get_conn,
    timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID,
    UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;

#[derive(Debug)]
struct ImmigrateParams {
    name: String,
    password: String,
    email: String,
}

impl ImmigrateParams {
    fn from_packet(packet: &AWPacket) -> Option<Self> {
        Some(ImmigrateParams {
            name: packet.get_string(VarID::CitizenName)?,
            password: packet.get_string(VarID::CitizenPassword)?,
            email: packet.get_string(VarID::CitizenEmail)?,
        })
    }
}

pub fn immigrate(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "immigrate");
    let mut response = AWPacket::new(PacketType::ImmigrateResponse);

    log::trace!("immigrate");

    let Some(params) = ImmigrateParams::from_packet(packet) else {
        return;
    };

    log::trace!("immigrate params {params:?}");

    let rc = try_immigrate(server, params)
        .err()
        .unwrap_or(ReasonCode::Success);

    response.add_uint(VarID::ReasonCode, rc.into());
    conn.send(response);
}

fn try_immigrate(server: &UniverseServer, params: ImmigrateParams) -> Result<(), ReasonCode> {
    if !server.config.allow_immigration {
        return Err(ReasonCode::ImmigrationNotAllowed);
    }

    check_valid_name(&params.name, false)?;
    check_valid_password(&params.password)?;
    // Normally, email is also validated, but I don't care about having valid emails.

    match server.database.citizen_by_name(&params.name) {
        DatabaseResult::Ok(Some(_)) => return Err(ReasonCode::NameAlreadyUsed),
        DatabaseResult::Ok(None) => {}
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    }

    let now = unix_epoch_timestamp_u32();

    let r = server.database.citizen_add_next(CitizenQuery {
        id: 0,
        changed: 0,
        name: params.name,
        password: params.password,
        email: params.email,
        priv_pass: String::new(),
        comment: String::new(),
        url: String::new(),
        immigration: now,
        expiration: 0,
        last_login: 0,
        last_address: 0,
        total_time: 0,
        bot_limit: 0,
        beta: 0,
        cav_enabled: 1,
        cav_template: 0,
        enabled: 1,
        privacy: 0,
        trial: 0,
    });

    match r {
        DatabaseResult::Ok(_) => Ok(()),
        DatabaseResult::DatabaseError => Err(ReasonCode::DatabaseError),
    }
}

fn check_valid_password(password: impl AsRef<str>) -> Result<(), ReasonCode> {
    let password = password.as_ref();
    if password.len() > 12 {
        return Err(ReasonCode::PasswordTooLong);
    }
    if password.len() < 4 {
        return Err(ReasonCode::PasswordTooShort);
    }

    Ok(())
}
