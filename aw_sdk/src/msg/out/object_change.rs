use aw_core::{AWPacket, PacketType, VarID};

use crate::{AwInstance, ObjectInfo, SdkError, SdkResult};

pub fn object_change(instance: &mut AwInstance, object_info: ObjectInfo) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::ObjectChange);
    let new_object_number = rand::random::<u32>();
    packet.add_uint(VarID::ObjectID, object_info.id);
    packet.add_uint(VarID::ObjectNumber, new_object_number);
    packet.add_uint(VarID::ObjectType, object_info.prop_type);
    packet.add_byte(VarID::ObjectSync, if object_info.sync { 1 } else { 0 });
    packet.add_int(VarID::ObjectWest, object_info.west);
    packet.add_int(VarID::ObjectHeight, object_info.height);
    packet.add_int(VarID::ObjectNorth, object_info.north);
    packet.add_int(VarID::ObjectRotation, object_info.rotation);
    packet.add_int(VarID::ObjectTilt, object_info.tilt);
    packet.add_int(VarID::ObjectRoll, object_info.roll);
    packet.add_uint(VarID::ObjectBuildTimestamp, object_info.build_timestamp);
    packet.add_uint(VarID::ObjectOwner, object_info.owner);
    packet.add_string(VarID::ObjectModel, object_info.model.to_string());
    packet.add_string(
        VarID::ObjectDescription,
        object_info.description.to_string(),
    );
    packet.add_string(VarID::ObjectAction, object_info.action.to_string());
    packet.add_data(VarID::ObjectData, object_info.data);

    if let Some(world) = &mut instance.world {
        world.connection.send(packet);
    } else {
        return Err(SdkError::NotConnectedToWorld);
    }
    Ok(())
}
