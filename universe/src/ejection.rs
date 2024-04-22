use std::net::IpAddr;

use aw_db::DatabaseResult;

use crate::{
    client::ClientInfo,
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

    let conn_serial = conn
        .client
        .as_ref()
        .and_then(ClientInfo::player_info)
        .and_then(|p| p.serial);

    // Safeguard Administrator from being ejected
    if let Some(user_id) = conn.client.as_ref().and_then(ClientInfo::citizen_id) {
        if user_id == 1 {
            log::trace!("Administrator would have been ejected, but has been prevented");
            return Some(false);
        }
    }

    let ip_u32 = u32::from_le_bytes(ip.octets());

    // Go through each ejection, check if IP or serial matches
    let mut eject_addr: u32 = 0;
    loop {
        // log::trace!("Checking ejection starting from {eject_addr}");
        match database.ejection_next(eject_addr) {
            DatabaseResult::Ok(Some(ejection)) => {
                eject_addr = ejection.address;

                // Don't enforce expired ejections
                if is_ejection_expired(&ejection) {
                    log::trace!("ejection expired");
                    continue;
                }

                // Does ejection match this connection's address?
                if ejection.address == ip_u32 {
                    log::trace!(
                        "Connection is ejected because it has address {ip:?} ({ip_u32}) - {conn:?}"
                    );
                    return Some(true);
                }

                // Get serial from ejection comment
                let comment = ejection.comment;
                let Some(comment_serial) = comment
                    .strip_prefix("serial=0x")
                    .and_then(|s| u32::from_str_radix(s, 16).ok())
                else {
                    log::trace!("no serial parsed in {comment:?}");
                    continue;
                };

                // log::trace!("Parsed serial {comment_serial:X} out of {comment:?}. conn's serial is {conn_serial:?}");

                // Is the serial in the comment the same as the conn's serial?
                if let Some(conn_serial) = conn_serial {
                    if conn_serial == comment_serial {
                        log::trace!(
                            "Connection is ejected because it has serial ({conn_serial:X}) - {conn:?}"
                        );
                        return Some(true);
                    }
                }
            }
            DatabaseResult::Ok(None) => return Some(false),
            DatabaseResult::DatabaseError => return None,
        }
    }

    // match database.ejection_lookup(ip_u32) {
    //     aw_db::DatabaseResult::Ok(Some(ejection)) => {
    //         if !is_ejection_expired(&ejection) {
    //             Some(true)
    //         } else {
    //             Some(false)
    //         }
    //     }
    //     aw_db::DatabaseResult::Ok(None) => Some(false),
    //     aw_db::DatabaseResult::DatabaseError => None,
    // }
}
