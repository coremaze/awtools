use crate::{
    attributes,
    attributes::set_attribute,
    client::{Client, ClientManager},
    database::Database,
};
use aw_core::*;

pub fn attribute_change(
    client: &Client,
    packet: &AWPacket,
    database: &Database,
    client_manager: &ClientManager,
) {
    // Only admins should be able to change Universe attributes
    if !client.has_admin_permissions() {
        log::info!(
            "Client {} tried to set attributes but is not an admin",
            client.addr.ip()
        );
        return;
    }

    for var in packet.get_vars().iter() {
        if let AWPacketVar::String(id, val) = var {
            log::info!("Client {} setting {:?} to {:?}", client.addr.ip(), id, val);
            set_attribute(*id, val, database).ok();
        }
    }

    for client in client_manager.clients() {
        attributes::send_attributes(client, database);
    }
}
