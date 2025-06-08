use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_object_click(
    instance: &mut AwInstance,
    packet: &AWPacket,
    events: &mut Vec<AwEvent>,
) {
    // let Ok(object_click_info) = ObjectClickInfo::try_from(packet) else {
    //     println!("Failed to parse object click: {packet:?}");
    //     return;
    // };
}

#[derive(Debug, Clone)]
pub struct ObjectClickInfo {
    pub avatar_session: u32,
    pub avatar_name: String,
    pub cell_x: i32,
    pub cell_z: i32,
    pub object_id: u32,
    pub object_number: u32,
    pub object_type: u32,
    pub object_sync: bool,
    pub object_west: i32,
    pub object_height: i32,
    pub object_north: i32,
    pub object_rotation: i32,
    pub object_tilt: i32,
    pub object_roll: i32,
    pub object_build_timestamp: u32,
    pub object_owner: u32,
    pub object_model: String,
    pub object_description: String,
    pub object_action: String,
    pub object_data: Vec<u8>,
}

// impl TryFrom<&AWPacket> for ObjectClickInfo {
//     type Error = SdkError;

//     fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
//         let Some(avatar_session) = packet.get_uint(VarID::MySession) else {
//             return Err(SdkError::missing_field("MySession"));
//         };
//         let Some(avatar_name) = packet.get_string(VarID::MyName) else {
//             return Err(SdkError::missing_field("MyName"));
//         };
//         let Some(cell_x) = packet.get_int(VarID::PositionWest) else {
//             return Err(SdkError::missing_field("PositionWest"));
//         };
//         let Some(cell_z) = packet.get_int(VarID::PositionHeight) else {
//             return Err(SdkError::missing_field("PositionHeight"));
//         };
//     }
// }
