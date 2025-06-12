pub mod attributes;
pub mod avatar_add;
pub mod avatar_change;
pub mod avatar_delete;
pub mod heartbeat;
pub mod message;
pub mod object_bump;
pub mod object_click;

use crate::SdkError;
use aw_core::{AWPacket, VarID};

#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub cell_x: i32,
    pub cell_z: i32,
    pub id: u32,
    pub number: u32,
    pub prop_type: u32,
    pub sync: bool,
    pub west: i32,
    pub height: i32,
    pub north: i32,
    pub rotation: i32,
    pub tilt: i32,
    pub roll: i32,
    pub build_timestamp: u32,
    pub owner: u32,
    pub model: String,
    pub description: String,
    pub action: String,
    pub data: Vec<u8>,
}

impl TryFrom<&AWPacket> for ObjectInfo {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        let cell_x = packet.get_int(VarID::ObjectCellX).unwrap_or(0);
        // Object cell z may default to 0 if not set
        let cell_z = packet.get_int(VarID::ObjectCellZ).unwrap_or(0);
        let id = packet
            .get_uint(VarID::ObjectID)
            .ok_or_else(|| SdkError::missing_field("ObjectID"))?;
        let number = packet
            .get_uint(VarID::ObjectNumber)
            .ok_or_else(|| SdkError::missing_field("ObjectNumber"))?;
        // Object type may default to 0 if not set
        let prop_type = packet.get_uint(VarID::ObjectType).unwrap_or(0);
        // Object sync may default to 0 if not set
        let sync = packet.get_byte(VarID::ObjectSync).unwrap_or(0);
        // Object west may default to 0 if not set
        let west = packet.get_int(VarID::ObjectWest).unwrap_or(0);
        // Object height may default to 0 if not set
        let height = packet.get_int(VarID::ObjectHeight).unwrap_or(0);
        // Object north may default to 0 if not set
        let north = packet.get_int(VarID::ObjectNorth).unwrap_or(0);
        // Object rotation may default to 0 if not set
        let rotation = packet.get_int(VarID::ObjectRotation).unwrap_or(0);
        // Object tilt may default to 0 if not set
        let tilt = packet.get_int(VarID::ObjectTilt).unwrap_or(0);
        // Object roll may default to 0 if not set
        let roll = packet.get_int(VarID::ObjectRoll).unwrap_or(0);
        let build_timestamp = packet
            .get_uint(VarID::ObjectBuildTimestamp)
            .ok_or_else(|| SdkError::missing_field("ObjectBuildTimestamp"))?;
        let owner = packet
            .get_uint(VarID::ObjectOwner)
            .ok_or_else(|| SdkError::missing_field("ObjectOwner"))?;
        let model = packet
            .get_string(VarID::ObjectModel)
            .ok_or_else(|| SdkError::missing_field("ObjectModel"))
            .unwrap_or("".to_string());
        // Object description may default to empty string if not set
        let description = packet
            .get_string(VarID::ObjectDescription)
            .unwrap_or("".to_string());
        // Object action may default to empty string if not set
        let action = packet
            .get_string(VarID::ObjectAction)
            .unwrap_or("".to_string());
        // Object data may default to empty vector if not set
        let data = packet.get_data(VarID::ObjectData).unwrap_or_default();

        Ok(ObjectInfo {
            cell_x,
            cell_z,
            id,
            number,
            prop_type,
            sync: sync != 0,
            west,
            height,
            north,
            rotation,
            tilt,
            roll,
            build_timestamp,
            owner,
            model,
            description,
            action,
            data,
        })
    }
}
