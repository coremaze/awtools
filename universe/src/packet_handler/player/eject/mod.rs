mod eject_add;
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;
pub use eject_add::eject_add;

mod eject_lookup;
pub use eject_lookup::eject_lookup;

mod eject_next;
pub use eject_next::eject_next;

mod eject_prev;
pub use eject_prev::eject_prev;

mod eject_delete;
pub use eject_delete::eject_delete;

use crate::{
    database::EjectDB, get_conn, universe_connection::UniverseConnectionID, UniverseServer,
};

enum EjectionLookupMethod {
    Previous,
    Exact,
    Next,
}

fn eject_lookup_by_method(
    server: &UniverseServer,
    cid: UniverseConnectionID,
    address: u32,
    method: EjectionLookupMethod,
) {
    let conn = get_conn!(server, cid, "eject_lookup_by_method");

    if !conn.has_admin_permissions() {
        log::trace!("eject_lookup_by_method failed because the client did not have permission");
        return;
    }

    let mut response = AWPacket::new(PacketType::EjectionInfo);

    let db_result = match method {
        EjectionLookupMethod::Previous => server.database.ejection_prev(address),
        EjectionLookupMethod::Exact => server.database.ejection_lookup(address),
        EjectionLookupMethod::Next => server.database.ejection_next(address),
    };

    let rc = match db_result {
        DatabaseResult::Ok(Some(ejection)) => {
            response.add_uint(VarID::EjectionAddress, ejection.address);
            response.add_uint(VarID::EjectionExpiration, ejection.expiration);
            response.add_uint(VarID::EjectionCreation, ejection.creation);
            response.add_string(VarID::EjectionComment, ejection.comment);

            ReasonCode::Success
        }
        DatabaseResult::Ok(None) => ReasonCode::NoSuchEjection,
        DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    };

    response.add_uint(VarID::ReasonCode, rc.into());
    conn.send(response);
}
