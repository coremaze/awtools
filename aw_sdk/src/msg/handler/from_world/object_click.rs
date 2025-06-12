use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError, msg::handler::from_world::ObjectInfo};

pub fn handle_object_click(
    instance: &mut AwInstance,
    packet: &AWPacket,
    events: &mut Vec<AwEvent>,
) {
    match ObjectClickInfo::try_from(packet) {
        Ok(object_click_info) => {
            events.push(AwEvent::ObjectClick(object_click_info));
        }
        Err(e) => {
            println!("Failed to parse object click: {packet:?}, {e}");
            return;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectClickInfo {
    pub avatar_session: u32,
    pub avatar_name: String,
    pub object_info: ObjectInfo,
}

impl TryFrom<&AWPacket> for ObjectClickInfo {
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
