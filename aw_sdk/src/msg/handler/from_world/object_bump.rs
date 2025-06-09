use aw_core::{AWPacket, VarID};

use crate::{AwEvent, AwInstance, SdkError};

pub fn handle_object_bump(instance: &mut AwInstance, packet: &AWPacket, events: &mut Vec<AwEvent>) {
    let object_bump_info = match ObjectBumpInfo::try_from(packet) {
        Ok(object_bump_info) => {
            events.push(AwEvent::ObjectBump(object_bump_info));
        }
        Err(e) => {
            println!("Failed to parse object bump: {packet:?}, {e}");
            return;
        }
    };
}

#[derive(Debug, Clone)]
pub struct ObjectBumpInfo {
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

impl TryFrom<&AWPacket> for ObjectBumpInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        let avatar_session = packet
            .get_uint(VarID::MySession)
            .ok_or_else(|| SdkError::missing_field("MySession"))?;
        let avatar_name = packet
            .get_string(VarID::MyName)
            .ok_or_else(|| SdkError::missing_field("MyName"))?;
        // Object cell x may default to 0 if not set
        let cell_x = packet.get_int(VarID::ObjectCellX).unwrap_or(0);
        // Object cell z may default to 0 if not set
        let cell_z = packet.get_int(VarID::ObjectCellZ).unwrap_or(0);
        let object_id = packet
            .get_uint(VarID::ObjectID)
            .ok_or_else(|| SdkError::missing_field("ObjectID"))?;
        let object_number = packet
            .get_uint(VarID::ObjectNumber)
            .ok_or_else(|| SdkError::missing_field("ObjectNumber"))?;
        // Object type may default to 0 if not set
        let object_type = packet.get_uint(VarID::ObjectType).unwrap_or(0);
        // Object sync may default to 0 if not set
        let object_sync = packet.get_byte(VarID::ObjectSync).unwrap_or(0);
        // Object west may default to 0 if not set
        let object_west = packet.get_int(VarID::ObjectWest).unwrap_or(0);
        // Object height may default to 0 if not set
        let object_height = packet.get_int(VarID::ObjectHeight).unwrap_or(0);
        // Object north may default to 0 if not set
        let object_north = packet.get_int(VarID::ObjectNorth).unwrap_or(0);
        // Object rotation may default to 0 if not set
        let object_rotation = packet.get_int(VarID::ObjectRotation).unwrap_or(0);
        // Object tilt may default to 0 if not set
        let object_tilt = packet.get_int(VarID::ObjectTilt).unwrap_or(0);
        // Object roll may default to 0 if not set
        let object_roll = packet.get_int(VarID::ObjectRoll).unwrap_or(0);
        let object_build_timestamp = packet
            .get_uint(VarID::ObjectBuildTimestamp)
            .ok_or_else(|| SdkError::missing_field("ObjectBuildTimestamp"))?;
        let object_owner = packet
            .get_uint(VarID::ObjectOwner)
            .ok_or_else(|| SdkError::missing_field("ObjectOwner"))?;
        let object_model = packet
            .get_string(VarID::ObjectModel)
            .ok_or_else(|| SdkError::missing_field("ObjectModel"))?;
        // Object description may default to empty string if not set
        let object_description = packet
            .get_string(VarID::ObjectDescription)
            .unwrap_or("".to_string());
        // Object action may default to empty string if not set
        let object_action = packet
            .get_string(VarID::ObjectAction)
            .unwrap_or("".to_string());
        // Object data may default to empty vector if not set
        let object_data = packet.get_data(VarID::ObjectData).unwrap_or_default();

        Ok(ObjectBumpInfo {
            avatar_session,
            avatar_name,
            cell_x,
            cell_z,
            object_id,
            object_number,
            object_type,
            object_sync: object_sync != 0,
            object_west,
            object_height,
            object_north,
            object_rotation,
            object_tilt,
            object_roll,
            object_build_timestamp,
            object_owner,
            object_model,
            object_description,
            object_action,
            object_data,
        })
    }
}
