use aw_core::{AWPacket, PacketType, PacketTypeResult};

use crate::instance_conn::AwInstanceConnection;
use crate::msg;
use crate::msg::handler::from_world::attributes::WorldAttributes;
use crate::{AwEvent, AwInstance};

pub struct World {
    pub connection: AwInstanceConnection,
    pub attributes: Option<WorldAttributes>,
}

pub fn handle_world_packet(instance: &mut AwInstance, packet: AWPacket, events: &mut Vec<AwEvent>) {
    let packet_type = packet.get_type();
    let packet_type = match packet_type {
        PacketTypeResult::Unknown(opcode) => {
            println!("Unknown packet opcode: {opcode}");
            return;
        }
        PacketTypeResult::PacketType(packet_type) => packet_type,
    };

    let handler = match packet_type {
        PacketType::Heartbeat => msg::handler::from_world::heartbeat::handle_heartbeat,
        PacketType::Message => msg::handler::from_world::message::handle_message,
        PacketType::AvatarChange => msg::handler::from_world::avatar_change::handle_avatar_change,
        PacketType::Attributes => msg::handler::from_world::attributes::handle_attributes,
        PacketType::AvatarAdd => msg::handler::from_world::avatar_add::handle_avatar_add,
        PacketType::AvatarDelete => msg::handler::from_world::avatar_delete::handle_avatar_delete,
        _ => {
            println!("Unhandled packet type: {packet_type:?}: {packet:?}",);
            return;
        }
    };

    handler(instance, &packet, events);
}
