use aw_core::{AWPacket, PacketType, VarID};

use crate::{AwInstance, SdkResult};

pub fn console_message(instance: &mut AwInstance, params: ConsoleMessageParams) -> SdkResult<()> {
    let mut packet = AWPacket::new(PacketType::ConsoleMessage);
    packet.add_uint(VarID::MySession, params.session_id);
    packet.add_string(VarID::ChatMessage, params.message);
    packet.add_byte(VarID::ConsoleBold, params.bold as u8);
    packet.add_byte(VarID::ConsoleItalics, params.italics as u8);
    packet.add_uint(VarID::ConsoleRed, params.color.0 as u32);
    packet.add_uint(VarID::ConsoleGreen, params.color.1 as u32);
    packet.add_uint(VarID::ConsoleBlue, params.color.2 as u32);

    if let Some(world) = &mut instance.world {
        world.connection.send(packet);
    }

    Ok(())
}

pub struct ConsoleMessageParams {
    pub session_id: u32,
    pub message: String,
    pub bold: bool,
    pub italics: bool,
    pub color: (u8, u8, u8),
}
