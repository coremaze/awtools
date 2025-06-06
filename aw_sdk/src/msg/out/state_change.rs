use aw_core::{AWPacket, PacketType, VarID};

use crate::{AwInstance, SdkError, SdkResult};

pub fn state_change(instance: &mut AwInstance, params: StateChangeParams) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::StateChange);
    packet.add_uint(VarID::PositionNorth, params.north);
    packet.add_uint(VarID::PositionHeight, params.height);
    packet.add_uint(VarID::PositionWest, params.west);
    packet.add_uint(VarID::PositionRotation, params.rotation);
    packet.add_uint(VarID::MyGesture, params.gesture);
    packet.add_uint(VarID::MyType, params.av_type);
    packet.add_uint(VarID::MyState, params.av_state);

    match instance.world {
        Some(ref mut world) => {
            world.connection.send(packet);
            Ok(())
        }
        None => Err(SdkError::NotConnectedToWorld),
    }
}

pub struct StateChangeParams {
    pub north: u32,    // x
    pub height: u32,   // y
    pub west: u32,     // z
    pub rotation: u32, // yaw
    pub gesture: u32,
    pub av_type: u32,
    pub av_state: u32,
}
