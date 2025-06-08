use crate::msg::handler::from_world::{
    avatar_add::AvatarAddInfo, avatar_change::AvatarChangeInfo, avatar_delete::AvatarDeleteInfo,
    message::MessageInfo,
};

#[derive(Debug)]
pub enum AwEvent {
    Message(MessageInfo),
    AvatarChange(AvatarChangeInfo),
    AvatarAdd(AvatarAddInfo),
    AvatarDelete(AvatarDeleteInfo),
}
