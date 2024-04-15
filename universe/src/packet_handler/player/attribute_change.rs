use crate::{attributes, get_conn, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::*;

pub fn attribute_change(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let conn = get_conn!(server, cid, "attribute_change");

    log::trace!("Changing attributes: {packet:?}");

    // Only admins should be able to change Universe attributes
    if !conn.has_admin_permissions() {
        log::info!(
            "Client {} tried to set attributes but is not an admin",
            conn.addr().ip()
        );
        return;
    }

    log::trace!("User is admin");

    // Set each of the received attributes
    for var in packet.get_vars().iter() {
        if let AWPacketVar::String(id, val) = var {
            log::info!("Client {} setting {:?} to {:?}", conn.addr().ip(), id, val);
            attributes::set_attribute(*id, val, &server.database);
        }
    }

    log::trace!("Updating attributes for everyone");

    // Update the attribtues for all connections
    for (_, other_conn) in server.connections.iter() {
        attributes::send_attributes(other_conn, &server.database);
    }

    let mut response = AWPacket::new(PacketType::AttributeChange);
    response.add_uint(VarID::ReasonCode, ReasonCode::Success.into());
    conn.send(response);
}
