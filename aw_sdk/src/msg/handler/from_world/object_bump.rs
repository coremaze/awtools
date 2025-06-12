use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError, msg::handler::from_world::ObjectInfo};

pub fn handle_object_bump(instance: &mut AwInstance, packet: &AWPacket, events: &mut Vec<AwEvent>) {
    match ObjectBumpInfo::try_from(packet) {
        Ok(object_bump_info) => {
            events.push(AwEvent::ObjectBump(object_bump_info));
        }
        Err(e) => {
            println!("Failed to parse object bump: {packet:?}, {e}");
            return;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectBumpInfo {
    pub avatar_session: u32,
    pub avatar_name: String,
    pub object_info: ObjectInfo,
}

impl TryFrom<&AWPacket> for ObjectBumpInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        let avatar_session = packet
            .get_uint(VarID::MySession)
            .ok_or_else(|| SdkError::missing_field("MySession"))?;
        let avatar_name = packet
            .get_string(VarID::MyName)
            .ok_or_else(|| SdkError::missing_field("MyName"))?;
        Ok(Self {
            avatar_session,
            avatar_name,
            object_info: ObjectInfo::try_from(packet)?,
        })
    }
}
