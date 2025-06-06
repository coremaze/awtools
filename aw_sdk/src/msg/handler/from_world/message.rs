use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_message(_instance: &mut AwInstance, packet: &AWPacket, events: &mut Vec<AwEvent>) {
    let Ok(message_info) = MessageInfo::try_from(packet) else {
        println!("Failed to parse message: {packet:?}");
        return;
    };
    events.push(AwEvent::Message(message_info));
}

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub message: String,
    pub chat_type: u32,
    pub name: String,
    pub user_session: u32,
}

impl TryFrom<&AWPacket> for MessageInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        let Some(message) = packet.get_string(VarID::ChatMessage) else {
            return Err(SdkError::missing_field("ChatMessage"));
        };
        let Some(chat_type) = packet.get_uint(VarID::ChatType) else {
            return Err(SdkError::missing_field("ChatType"));
        };
        let Some(name) = packet.get_string(VarID::MyName) else {
            return Err(SdkError::missing_field("MyName"));
        };
        let Some(user_session) = packet.get_uint(VarID::MySession) else {
            return Err(SdkError::missing_field("MySession"));
        };
        Ok(Self {
            message,
            chat_type,
            name,
            user_session,
        })
    }
}
