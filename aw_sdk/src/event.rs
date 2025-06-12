use crate::msg::handler::from_world::{
    ObjectInfo, avatar_add::AvatarAddInfo, avatar_change::AvatarChangeInfo,
    avatar_delete::AvatarDeleteInfo, message::MessageInfo, object_bump::ObjectBumpInfo,
    object_click::ObjectClickInfo,
};

#[derive(Debug)]
pub enum AwEvent {
    UniverseDisconnected,
    WorldDisconnected,
    Message(MessageInfo),
    AvatarChange(AvatarChangeInfo),
    AvatarAdd(AvatarAddInfo),
    AvatarDelete(AvatarDeleteInfo),
    ObjectClick(ObjectClickInfo),
    ObjectBump(ObjectBumpInfo),
}
