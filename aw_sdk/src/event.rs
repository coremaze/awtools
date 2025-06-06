use crate::msg::handler::from_world::{avatar_change::AvatarChangeInfo, message::MessageInfo};

pub enum AwEvent {
    Message(MessageInfo),
    AvatarChange(AvatarChangeInfo),
}
