use crate::{
    client::ClientInfo, database::ContactDB, get_conn, tabs::regenerate_contact_list,
    universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::*;
use aw_db::DatabaseResult;

pub fn contact_delete(server: &mut UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let Some(other_cit_id) = packet.get_uint(VarID::ContactListCitizenID) else {
        return;
    };

    let conn = get_conn!(server, cid, "contact_delete");

    let Some(self_cit_id) = conn.client.as_ref().and_then(ClientInfo::citizen_id) else {
        return;
    };

    let mut rc = ReasonCode::Success;
    match server.database.contact_blocked(other_cit_id, self_cit_id) {
        DatabaseResult::Ok(blocked_by_other_person) => {
            match server.database.contact_delete(self_cit_id, other_cit_id) {
                DatabaseResult::Ok(_) => {}
                DatabaseResult::DatabaseError => rc = ReasonCode::UnableToSetContact,
            }

            if !blocked_by_other_person {
                match server.database.contact_delete(other_cit_id, self_cit_id) {
                    DatabaseResult::Ok(_) => {}
                    DatabaseResult::DatabaseError => rc = ReasonCode::DatabaseError,
                }
            }
        }
        DatabaseResult::DatabaseError => rc = ReasonCode::DatabaseError,
    }

    let mut response = AWPacket::new(PacketType::ContactDelete);
    response.add_uint(VarID::ReasonCode, rc.into());
    conn.send(response);

    regenerate_contact_list(server, cid);

    if let Some(other_cid) = server.connections.get_by_citizen_id(other_cit_id) {
        regenerate_contact_list(server, other_cid);
    }
}
