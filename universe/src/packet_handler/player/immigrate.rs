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

#[derive(Debug)]
enum ImmigrateParamsError {
    Name,
    Password,
    Email,
}

impl TryFrom<&AWPacket> for ImmigrateParams {
    type Error = ImmigrateParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let name = value
            .get_string(VarID::CitizenName)
            .ok_or(ImmigrateParamsError::Name)?;
        let password = value
            .get_string(VarID::CitizenPassword)
            .ok_or(ImmigrateParamsError::Password)?;
        let email = value
            .get_string(VarID::CitizenEmail)
            .ok_or(ImmigrateParamsError::Email)?;

        Ok(Self {
            name,
            password,
            email,
        })
    }
}

pub fn immigrate(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "immigrate");
    let mut response = AWPacket::new(PacketType::ImmigrateResponse);

    log::trace!("immigrate");

    let params = match ImmigrateParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete immigrate: {why:?}");
            return;
        }
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
