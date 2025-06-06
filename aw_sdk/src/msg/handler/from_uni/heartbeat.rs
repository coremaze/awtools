use aw_core::AWPacket;

use crate::{AwEvent, AwInstance, msg};

pub fn handle_heartbeat(instance: &mut AwInstance, _packet: &AWPacket, _events: &mut Vec<AwEvent>) {
    msg::out::heartbeat::uni_heartbeat(instance);
}
