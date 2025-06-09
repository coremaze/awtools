use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_avatar_change(
    _instance: &mut AwInstance,
    packet: &AWPacket,
    events: &mut Vec<AwEvent>,
) {
    match AvatarChangeInfo::try_from(packet) {
        Ok(avatar_change_info) => {
            events.push(AwEvent::AvatarChange(avatar_change_info));
        }
        Err(e) => {
            println!("Failed to parse avatar change info: {e:?} {packet:?}");
            return;
        }
    };
}

#[derive(Debug, Clone)]
pub struct AvatarChangeInfo {
    pub session_id: u32,
    pub name: String,
    pub north: i32,    // x
    pub height: i32,   // y
    pub west: i32,     // z
    pub rotation: i32, // yaw
}

impl TryFrom<&AWPacket> for AvatarChangeInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            session_id: packet
                .get_uint(VarID::MySession)
                .ok_or(SdkError::missing_field("MySession"))?,
            name: packet
                .get_string(VarID::MyName)
                .ok_or(SdkError::missing_field("MyName"))?,
            north: packet.get_int(VarID::PositionNorth).unwrap_or(0),
            height: packet.get_int(VarID::PositionHeight).unwrap_or(0),
            west: packet.get_int(VarID::PositionWest).unwrap_or(0),
            rotation: packet.get_int(VarID::PositionRotation).unwrap_or(0),
        })
    }
}
