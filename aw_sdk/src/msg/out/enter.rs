use aw_core::{AWPacket, PacketType, VarID};

use crate::{
    AwInstance, SdkError, SdkResult, instance_conn::AwInstanceConnection,
    msg::handler::from_world::attributes::WorldAttributes, world::World,
};

pub fn enter(instance: &mut AwInstance, world: &str, global: bool) -> SdkResult<()> {
    instance.world = None;

    let world_info = instance.world_lookup(world)?;
    // eprintln!("World info: {world_info:?}");

    let address_octets = world_info.address.to_le_bytes();
    let address = format!(
        "{}.{}.{}.{}",
        address_octets[0], address_octets[1], address_octets[2], address_octets[3]
    );
    let mut world_conn = AwInstanceConnection::connect(&address, world_info.port)?;

    let mut enter_packet = AWPacket::new(PacketType::Enter);

    let session = instance.session.as_ref().ok_or(SdkError::NotLoggedIn)?;

    let session_id = session.session_id;
    let login_id = session.login_id;

    enter_packet.add_string(VarID::WorldName, world_info.name);
    enter_packet.add_data(VarID::WorldUserNonce, world_info.world_user_nonce);
    enter_packet.add_uint(VarID::SessionID, session_id);
    enter_packet.add_uint(VarID::LoginID, login_id.unwrap_or(0));
    enter_packet.add_byte(VarID::WorldEnterGlobal, global as u8);
    enter_packet.add_int(VarID::EventMask, 0x2f7b);
    world_conn.send(enter_packet);

    let response = world_conn
        .wait_for_packet(PacketType::Enter, None)
        .ok_or(SdkError::Timeout)?;
    eprintln!("Enter response: {response:?}");

    instance.world = Some(World {
        connection: world_conn,
        attributes: None,
    });
    Ok(())
}
