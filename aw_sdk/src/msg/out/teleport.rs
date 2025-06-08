use aw_core::{AWPacket, PacketType, VarID};

use crate::{AwInstance, SdkError, SdkResult};

pub fn teleport(instance: &mut AwInstance, params: TeleportParams) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::Teleport);
    packet.add_string(VarID::TeleportWorld, params.world.clone());
    packet.add_int(VarID::TeleportNorth, params.north);
    packet.add_int(VarID::TeleportHeight, params.height);
    packet.add_int(VarID::TeleportWest, params.west);
    packet.add_int(VarID::TeleportRotation, params.rotation);
    packet.add_byte(VarID::TeleportWarp, params.warp as u8);

    match instance.world {
        Some(ref mut world) => {
            world.connection.send(packet);
            Ok(())
        }
        None => Err(SdkError::NotConnectedToWorld),
    }
}

pub struct TeleportParams {
    pub session_id: u32, // The user to teleport
    pub world: String,
    pub north: i32,
    pub height: i32,
    pub west: i32,
    pub rotation: i32,
    pub warp: bool,
}
