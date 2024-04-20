use std::net::IpAddr;

use crate::{
    database::{eject::EjectionQuery, EjectDB, UniverseDatabase},
    timestamp::unix_epoch_timestamp_u32,
    UniverseConnection,
};

pub fn is_ejection_expired(ejection: &EjectionQuery) -> bool {
    log::trace!(
        "unix_epoch_timestamp_u32() = {}; ejection.expiration = {}",
        unix_epoch_timestamp_u32(),
        ejection.expiration
    );
    unix_epoch_timestamp_u32() > ejection.expiration
}

pub fn is_connection_ejected(
    database: &UniverseDatabase,
    conn: &UniverseConnection,
) -> Option<bool> {
    let IpAddr::V4(ip) = conn.addr().ip() else {
        return Some(false);
    };

    let ip_u32 = u32::from_le_bytes(ip.octets());

    match database.ejection_lookup(ip_u32) {
        aw_db::DatabaseResult::Ok(Some(ejection)) => {
            if !is_ejection_expired(&ejection) {
                Some(true)
            } else {
                Some(false)
            }
        }
        aw_db::DatabaseResult::Ok(None) => Some(false),
        aw_db::DatabaseResult::DatabaseError => None,
    }
}
