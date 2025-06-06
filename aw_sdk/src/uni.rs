use aw_core::{AWPacket, PacketType, PacketTypeResult};

use crate::msg;
use crate::{AwEvent, instance::AwInstance};
pub fn handle_uni_packet(instance: &mut AwInstance, packet: AWPacket, events: &mut Vec<AwEvent>) {
    let packet_type = packet.get_type();
    let packet_type = match packet_type {
        PacketTypeResult::Unknown(opcode) => {
            println!("Unknown packet opcode: {opcode}");
            return;
        }
        PacketTypeResult::PacketType(packet_type) => packet_type,
    };

    let handler = match packet_type {
        PacketType::Heartbeat => msg::handler::from_uni::heartbeat::handle_heartbeat,
        _ => {
            println!("Unhandled packet type: {packet_type:?}: {packet:?}",);
            return;
        }
    };

    handler(instance, &packet, events);
}
