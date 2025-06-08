use aw_core::{AWPacket, AWPacketVar, PacketType, VarID};

use crate::{AwInstance, SdkError, SdkResult};

pub fn hud_create(
    instance: &mut AwInstance,
    params: HudCreateParams,
) -> SdkResult<HudCreateResult> {
    let _ = instance;
    let mut packet = AWPacket::new(PacketType::HudCreate);
    let hud_element_type: u32 = match &params.element_type {
        HudType::Text { .. } => 0,
        HudType::Image { .. } => 1,
    };

    packet.add_uint(VarID::HudElementType, hud_element_type);
    packet.add_uint(VarID::HudElementId, params.element_id);
    packet.add_uint(VarID::HudElementSession, params.user_session);
    packet.add_uint(VarID::HudElementOrigin, params.element_origin as u32);
    packet.add_float(VarID::HudElementOpacity, params.element_opacity);
    packet.add_int(VarID::HudElementX, params.element_x);
    packet.add_int(VarID::HudElementY, params.element_y);
    packet.add_int(VarID::HudElementZ, params.element_z);
    packet.add_uint(
        VarID::HudElementFlags,
        params
            .element_flags
            .iter()
            .fold(0, |acc, flag| acc | *flag as u32),
    );

    let element_text = match &params.element_type {
        HudType::Text { element_text } => element_text.clone(),
        HudType::Image { texture_name, .. } => texture_name.clone(),
    };
    packet.add_string(VarID::HudElementText, element_text);

    let element_color = params.element_color.0 as u32
        | (params.element_color.1 as u32) << 8
        | (params.element_color.2 as u32) << 16;

    packet.add_uint(VarID::HudElementColor, element_color);
    packet.add_uint(VarID::HudElementSizeX, params.element_size_x);
    packet.add_uint(VarID::HudElementSizeY, params.element_size_y);
    packet.add_uint(VarID::HudElementSizeZ, params.element_size_z);

    if let HudType::Image {
        texture_offset_x,
        texture_offset_y,
        ..
    } = &params.element_type
    {
        packet.add_uint(VarID::HudElementTextureOffsetX, *texture_offset_x);
        packet.add_uint(VarID::HudElementTextureOffsetY, *texture_offset_y);
    } else {
        packet.add_uint(VarID::HudElementTextureOffsetX, 0);
        packet.add_uint(VarID::HudElementTextureOffsetY, 0);
    }

    let result = match &mut instance.world {
        Some(world) => {
            println!("Sending hud create packet");
            println!("Packet: {:?}", &packet);
            world.connection.send(packet);
            world
                .connection
                .wait_for_packet(PacketType::HudResult, Some(instance.timeout))
                .ok_or(SdkError::Timeout)?
        }
        None => return SdkResult::Err(SdkError::NotConnectedToWorld),
    };

    let result = HudCreateResult::try_from(&result)?;

    Ok(result)
}

pub fn hud_clear() -> SdkResult<()> {
    Ok(())
}

pub struct HudCreateParams {
    pub element_type: HudType,
    pub element_id: u32,
    pub user_session: u32,
    pub element_origin: HudOrigin,
    pub element_opacity: f32,
    pub element_x: i32,
    pub element_y: i32,
    pub element_z: i32,
    pub element_flags: Vec<HudElementFlags>,
    pub element_color: (u8, u8, u8),
    pub element_size_x: u32,
    pub element_size_y: u32,
    pub element_size_z: u32,
}

pub enum HudOrigin {
    TopLeft = 0,
    Top = 1,
    TopRight = 2,
    Left = 3,
    Center = 4,
    Right = 5,
    BottomLeft = 6,
    Bottom = 7,
    BottomRight = 8,
}

pub enum HudType {
    Text {
        // 1
        element_text: String,
    },
    Image {
        // 2
        texture_name: String,
        texture_offset_x: u32,
        texture_offset_y: u32,
    },
    // Model{name: String}, // 3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudElementFlags {
    Clicks = 0x0001,
    Stretch = 0x0002,
    Additive = 0x0004,
    SizePercent = 0x0008,
    Transition = 0x0010,
    Temporary = 0x0020,
    UniversePath = 0x0040,
    Clamp = 0x0080,
    Highlight = 0x0100,
}

pub struct HudCreateResult {
    pub element_id: u32,
    pub user_session: u32,
}

impl TryFrom<&AWPacket> for HudCreateResult {
    type Error = SdkError;

    fn try_from(packet: &AWPacket) -> Result<Self, Self::Error> {
        Ok(Self {
            element_id: packet
                .get_uint(VarID::HudElementId)
                .ok_or(SdkError::missing_field("HudElementId"))?,
            user_session: packet
                .get_uint(VarID::HudElementSession)
                .ok_or(SdkError::missing_field("HudElementSession"))?,
        })
    }
}
