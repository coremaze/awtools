use aw_core::{AWPacket, PacketType, VarID};

use crate::{AwInstance, SdkError, SdkResult};

pub fn say(instance: &mut AwInstance, message: &str) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::Message);
    packet.add_string(VarID::ChatMessage, message.to_string());
    match instance.world {
        Some(ref mut world) => {
            world.connection.send(packet);
            Ok(())
        }
        None => Err(SdkError::NotConnectedToWorld),
    }
}
