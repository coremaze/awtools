use crate::{
    database::{contact::ContactOptions, CitizenDB, ContactDB, TelegramDB, UniverseDatabase},
    get_conn,
    tabs::regenerate_contact_list,
    telegram,
    timestamp::unix_epoch_timestamp_u32,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

#[derive(Debug)]
enum ContactAddParamsError {
    ContactName,
    ContactOptions,
}

struct ContactAddParams {
    contact_name: String,
    contact_options: ContactOptions,
}

impl TryFrom<&AWPacket> for ContactAddParams {
    type Error = ContactAddParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let contact_name = value
            .get_string(VarID::ContactListName)
            .ok_or(ContactAddParamsError::ContactName)?;

        let contact_options = value
            .get_uint(VarID::ContactListOptions)
            .map(ContactOptions::from_bits_truncate)
            .ok_or(ContactAddParamsError::ContactOptions)?;

        Ok(Self {
            contact_name,
            contact_options,
        })
    }
}

pub fn contact_add(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let params = match ContactAddParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete contact add: {why:?}");
            return;
        }
    };

    let mut response = AWPacket::new(PacketType::ContactAdd);
    let conn = get_conn!(server, cid, "contact_add");

    let rc = match try_add_contact(conn, params, &server.database) {
        Ok((cit_id, cont_id)) => {
            alert_friend_request(cit_id, cont_id, server);
            response.add_uint(VarID::ContactListCitizenID, cont_id);
            // response.add_uint(
            //     VarID::ContactListOptions,
            //     server.database.contact_get_default(cit_id).bits(),
            // );

            ReasonCode::Success
        }
        Err(x) => x,
    };

    log::info!("Contact add: {rc:?}");
    response.add_int(VarID::ReasonCode, rc as i32);

    conn.send(response);
    regenerate_contact_list(server, cid);
}

fn alert_friend_request(from: u32, to: u32, server: &UniverseServer) {
    let now = unix_epoch_timestamp_u32();

    let citizen = match server.database.citizen_by_number(from) {
        DatabaseResult::Ok(Some(citizen)) => citizen,
        DatabaseResult::Ok(None) => return,
        DatabaseResult::DatabaseError => {
            log::error!("Could not complete alert_friend_request due to database error.");
            return;
        }
    };

    // Create a telegram to alert user of friend request
    let source_username = citizen.name;
    if server
        .database
        .telegram_add(to, from, now, &format!("\n\x01({from}){source_username}\n"))
        .is_err()
    {
        return;
    }

    // Alert recipient of new telegram
    if let Some(target_cid) = server.connections.get_by_citizen_id(to) {
        telegram::send_telegram_update_available(server, target_cid);
    }
}

fn try_add_contact(
    conn: &UniverseConnection,
    params: ContactAddParams,
    database: &UniverseDatabase,
) -> Result<(u32, u32), ReasonCode> {
    // Must be a player logged in as a citizen
    let client = conn.client.as_ref().ok_or(ReasonCode::NotLoggedIn)?;
    let citizen = client.citizen().ok_or(ReasonCode::NotLoggedIn)?;

    let citizen_id = citizen.cit_id;

    let contact_citizen = match database.citizen_by_name(&params.contact_name) {
        DatabaseResult::Ok(Some(cit)) => cit,
        DatabaseResult::Ok(None) => return Err(ReasonCode::NoSuchCitizen),
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    let other_has_blocked_you = match database.contact_blocked(contact_citizen.id, citizen_id) {
        DatabaseResult::Ok(blocked) => blocked,
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    if other_has_blocked_you && !params.contact_options.contains(ContactOptions::ALL_BLOCKED) {
        return Err(ReasonCode::ContactAddBlocked);
    }

    let source_has_contact = match database.contact_get(citizen_id, contact_citizen.id) {
        DatabaseResult::Ok(Some(_)) => true,
        DatabaseResult::Ok(None) => false,
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    let target_has_contact = match database.contact_get(contact_citizen.id, citizen_id) {
        DatabaseResult::Ok(Some(_)) => true,
        DatabaseResult::Ok(None) => false,
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    // Stop people from adding each other when they are already friends
    if source_has_contact && target_has_contact {
        // Haven't checked if this is the right error code to send
        return Err(ReasonCode::UnableToSetContact);
    }

    let mut options = params.contact_options;
    options.remove(ContactOptions::FRIEND_REQUEST_ALLOWED);
    options.insert(ContactOptions::FRIEND_REQUEST_BLOCKED);

    match database.contact_set(citizen_id, contact_citizen.id, options.bits()) {
        DatabaseResult::Ok(_) => Ok((citizen_id, contact_citizen.id)),
        DatabaseResult::DatabaseError => Err(ReasonCode::DatabaseError),
    }
}
