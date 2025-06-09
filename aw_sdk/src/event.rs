use crate::{
    ObjectBumpInfo,
    msg::handler::from_world::{
        avatar_add::AvatarAddInfo, avatar_change::AvatarChangeInfo,
        avatar_delete::AvatarDeleteInfo, message::MessageInfo, object_click::ObjectClickInfo,
    },
};

#[derive(Debug)]
pub enum AwEvent {
    Message(MessageInfo),
    AvatarChange(AvatarChangeInfo),
    AvatarAdd(AvatarAddInfo),
    AvatarDelete(AvatarDeleteInfo),
    ObjectClick(ObjectClickInfo),
    ObjectBump(ObjectBumpInfo),
}
