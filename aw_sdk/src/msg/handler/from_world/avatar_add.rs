use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_avatar_add(_instance: &mut AwInstance, packet: &AWPacket, events: &mut Vec<AwEvent>) {
    match AvatarAddInfo::try_from(packet) {
        Ok(avatar_add_info) => {
            events.push(AwEvent::AvatarAdd(avatar_add_info));
        }
        Err(e) => {
            println!("Failed to parse avatar add info: {e:?} {packet:?}");
            return;
        }
    };
}

#[derive(Debug, Clone)]
pub struct AvatarAddInfo {
    pub citizen_id: Option<u32>,
    pub session_id: u32,
    pub name: String,
    pub north: i32,
    pub height: i32,
    pub west: i32,
    pub rotation: i32,
    pub pitch: i32,
    pub state: Option<u32>,
    // Also has build and pluginstring
}

impl TryFrom<&AWPacket> for AvatarAddInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            citizen_id: packet.get_uint(VarID::MyID).filter(|&id| id != 0),
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
            pitch: packet.get_int(VarID::MyPitch).unwrap_or(0),
            state: packet.get_uint(VarID::MyState),
        })
    }
}
