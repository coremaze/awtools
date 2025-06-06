use aw_core::{AWPacket, PacketType};

use crate::AwInstance;

pub fn uni_heartbeat(instance: &mut AwInstance) {
    let packet = AWPacket::new(PacketType::Heartbeat);
    instance.uni.send(packet);
}

pub fn world_heartbeat(instance: &mut AwInstance) {
    let packet = AWPacket::new(PacketType::Heartbeat);
    if let Some(world) = &mut instance.world {
        world.connection.send(packet);
    }
}
