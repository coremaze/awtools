mod error;
mod event;
mod instance;
mod instance_conn;
mod msg;
mod object_pass;
mod uni;
mod world;

pub use object_pass::PkwareDcl;

pub use event::AwEvent;
pub use instance::AwInstance;

pub use error::{SdkError, SdkResult};

pub use msg::handler::from_world::{
    ObjectInfo, avatar_add::AvatarAddInfo, avatar_change::AvatarChangeInfo,
    avatar_delete::AvatarDeleteInfo, message::MessageInfo, object_bump::ObjectBumpInfo,
    object_click::ObjectClickInfo,
};
pub use msg::out::console_message::ConsoleMessageParams;
pub use msg::out::hud::{HudCreateParams, HudCreateResult, HudElementFlags, HudOrigin, HudType};
pub use msg::out::login::{LoginParams, LoginResult};
pub use msg::out::query::QueryResult;
pub use msg::out::state_change::StateChangeParams;
pub use msg::out::teleport::TeleportParams;
pub use msg::out::world_lookup::WorldInfo;

pub fn sector_from_cell(mut n: i32) -> i32 {
    if n < 0 {
        n -= 3;
    } else {
        n += 4;
    }

    if -1 < n {
        return n >> 3;
    }

    return (n + 7) >> 3;
}

pub fn cell_from_cm(mut n: i32) -> i32 {
    if n < 0 {
        n -= 999;
    }

    ((n as f32) * 0.001).round() as i32
}
