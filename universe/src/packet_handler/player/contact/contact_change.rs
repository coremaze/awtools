use crate::{
    client::ClientInfo,
    database::{contact::ContactOptions, ContactDB},
    get_conn,
    tabs::regenerate_contact_list_and_mutuals,
    universe_connection::UniverseConnectionID,
    UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

#[derive(Debug)]
enum ContactChangeParamsError {
    ContactCitizenID,
    OptionChanges,
}

struct ContactChangeParams {
    contact_citizen_id: u32,
    option_changes: ContactOptions,
}

impl TryFrom<&AWPacket> for ContactChangeParams {
    type Error = ContactChangeParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let contact_citizen_id = value
            .get_uint(VarID::ContactListCitizenID)
            .ok_or(ContactChangeParamsError::ContactCitizenID)?;

        let option_changes = value
            .get_uint(VarID::ContactListOptions)
            .and_then(ContactOptions::from_bits)
            .ok_or(ContactChangeParamsError::OptionChanges)?;

        Ok(Self {
            contact_citizen_id,
            option_changes,
        })
    }
}

pub fn contact_change(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let params = match ContactChangeParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete contact change: {why:?}");
            return;
        }
    };

    let Some(self_citizen_id) = get_conn!(server, cid, "contact_change")
        .client
        .as_ref()
        .and_then(ClientInfo::citizen_id)
    else {
        return;
    };

    let original_options = match server
        .database
        .contact_get(self_citizen_id, params.contact_citizen_id)
    {
        DatabaseResult::Ok(Some(q)) => q.options,
        // The user may not have an entry for a contact with 0 yet
        DatabaseResult::Ok(None) if params.contact_citizen_id == 0 => ContactOptions::empty(),
        DatabaseResult::Ok(None) => return,
        DatabaseResult::DatabaseError => {
            log::error!("Could not complete contact_change due to database error.");
            return;
        }
    };

    let new_options = original_options.apply_changes(params.option_changes);

    match server.database.contact_set(
        self_citizen_id,
        params.contact_citizen_id,
        new_options.bits(),
    ) {
        DatabaseResult::Ok(_) => {}
        DatabaseResult::DatabaseError => {
            log::error!("Could not complete contact_change due to database error.");
            return;
        }
    }

    if params.option_changes.contains(ContactOptions::ALL_BLOCKED) {
        match server
            .database
            .contact_delete(params.contact_citizen_id, self_citizen_id)
        {
            DatabaseResult::Ok(_) => {}
            DatabaseResult::DatabaseError => {
                log::error!("Could not complete contact_change due to database error.");
                return;
            }
        }
    }

    regenerate_contact_list_and_mutuals(server, cid);
}
