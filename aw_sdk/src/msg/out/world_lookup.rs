use aw_core::{AWPacket, PacketType, ReasonCode, VarID};

use crate::{AwInstance, SdkError, SdkResult};

pub fn world_lookup(instance: &mut AwInstance, world: &str) -> SdkResult<WorldInfo> {
    let mut packet = AWPacket::new(PacketType::WorldLookup);
    packet.add_string(VarID::WorldName, world.to_string());
    instance.uni.send(packet);

    let response = instance
        .uni
        .wait_for_packet(PacketType::WorldLookup, Some(instance.timeout))
        .ok_or(SdkError::Timeout)?;

    let reason_code = response
        .get_int(VarID::ReasonCode)
        .ok_or_else(|| SdkError::missing_field("ReasonCode"))?;

    let reason_code =
        ReasonCode::try_from(reason_code).map_err(|_| SdkError::protocol("Invalid reason code"))?;

    if reason_code != ReasonCode::Success {
        return Err(SdkError::ServerError(reason_code));
    }

    let name = response
        .get_string(VarID::WorldName)
        .ok_or_else(|| SdkError::missing_field("WorldName"))?;
    let address = response
        .get_uint(VarID::WorldAddress)
        .ok_or_else(|| SdkError::missing_field("WorldAddress"))?;
    let port = response
        .get_uint(VarID::WorldPort)
        .ok_or_else(|| SdkError::missing_field("WorldPort"))?;
    let world_license_users = response
        .get_uint(VarID::WorldLicenseUsers)
        .ok_or_else(|| SdkError::missing_field("WorldLicenseUsers"))?;
    let world_user_nonce = response
        .get_data(VarID::WorldUserNonce)
        .ok_or_else(|| SdkError::missing_field("WorldUserNonce"))?;

    let world_info = WorldInfo {
        name,
        address,
        port: port as u16,
        world_license_users,
        world_user_nonce,
    };
    Ok(world_info)
}

#[derive(Debug, Clone)]
pub struct WorldInfo {
    pub name: String,
    pub address: u32,
    pub port: u16,
    pub world_license_users: u32,
    pub world_user_nonce: Vec<u8>,
}
