use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_avatar_delete(
    _instance: &mut AwInstance,
    packet: &AWPacket,
    events: &mut Vec<AwEvent>,
) {
    match AvatarDeleteInfo::try_from(packet) {
        Ok(avatar_delete_info) => {
            events.push(AwEvent::AvatarDelete(avatar_delete_info));
        }
        Err(e) => {
            println!("Failed to parse avatar delete info: {e:?} {packet:?}");
            return;
        }
    };
}

#[derive(Debug, Clone)]
pub struct AvatarDeleteInfo {
    pub session_id: u32,
    pub name: String,
}

impl TryFrom<&AWPacket> for AvatarDeleteInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            session_id: packet
                .get_uint(VarID::MySession)
                .ok_or(SdkError::missing_field("MySession"))?,
            name: packet
                .get_string(VarID::MyName)
                .ok_or(SdkError::missing_field("MyName"))?,
        })
    }
}
