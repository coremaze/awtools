use crate::{
    database::{contact::ContactOptions, ContactDB, UniverseDatabase},
    get_conn,
    tabs::regenerate_contact_list_and_mutuals,
    universe_connection::UniverseConnectionID,
    UniverseConnection, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

pub fn contact_confirm(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "contact_confirm");

    let rc = match try_contact_confirm(conn, packet, &server.database) {
        Ok(_) => ReasonCode::Success,
        Err(x) => x,
    };

    let mut response = AWPacket::new(PacketType::ContactConfirm);
    response.add_int(VarID::ReasonCode, rc as i32);
    conn.send(response);
    regenerate_contact_list_and_mutuals(server, cid);
}

fn try_contact_confirm(
    conn: &UniverseConnection,
    packet: &AWPacket,
    database: &UniverseDatabase,
) -> Result<(), ReasonCode> {
    // Must be a player logged in as a citizen
    let client = conn.client.as_ref().ok_or(ReasonCode::NotLoggedIn)?;
    let citizen = client.citizen().ok_or(ReasonCode::NotLoggedIn)?;

    let citizen_id = citizen.cit_id;

    let contact_id = packet
        .get_uint(VarID::ContactListCitizenID)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    if packet.get_int(VarID::ContactListOptions).unwrap_or(-1) == -1 {
        // Contact request denied
        return Ok(());
    }

    let contact_options = packet
        .get_uint(VarID::ContactListOptions)
        .ok_or(ReasonCode::NoSuchCitizen)?;

    let target_options = match database.contact_get(contact_id, citizen_id) {
        DatabaseResult::Ok(Some(target)) => target.options,
        // Fail if the target has no contact for this citizen (i.e. this contact was not requested)
        DatabaseResult::Ok(None) => return Err(ReasonCode::UnableToSetContact),
        DatabaseResult::DatabaseError => return Err(ReasonCode::DatabaseError),
    };

    if !target_options.is_friend_request_allowed() {
        return Err(ReasonCode::UnableToSetContact);
    }

    // Fail if could not set the contacts
    if database.contact_set(citizen_id, contact_id, 0).is_err()
        || database.contact_set(contact_id, citizen_id, 0).is_err()
    {
        return Err(ReasonCode::UnableToSetContact);
    }

    log::info!(
        "Accepted contact {:?}",
        ContactOptions::from_bits_truncate(contact_options)
    );

    Ok(())
}
