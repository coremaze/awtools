//! Packet (de)serialization for AW
use crate::net::packet_var::{AWPacketVar, VarID};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Read, Write};

/// Packet which can be sent over an AWProtocol.
#[derive(Debug, PartialEq, Clone)]
pub struct AWPacket {
    vars: Vec<AWPacketVar>,
    opcode: PacketTypeResult,
    header_0: u16,
    header_1: u16,
}

impl AWPacket {
    /// Create a new packet with a given type.
    pub fn new(opcode: PacketType) -> Self {
        Self {
            vars: Vec::new(),
            opcode: PacketTypeResult::PacketType(opcode),
            header_0: 0, // Defaults to 2 in AW 6
            header_1: 2, // Defaults to 3 in AW 6
        }
    }

    /// Get the type of the packet.
    pub fn get_type(&self) -> PacketTypeResult {
        self.opcode
    }

    pub fn set_header_0(&mut self, header_0: u16) {
        self.header_0 = header_0;
    }

    pub fn set_header_1(&mut self, header_1: u16) {
        self.header_1 = header_1;
    }

    /// Add a variable to the packet.
    pub fn add_var(&mut self, var: AWPacketVar) {
        self.vars.push(var);
    }

    /// Get a variable from a packet.
    pub fn get_var(&self, var_id: VarID) -> Option<&AWPacketVar> {
        self.vars.iter().find(|&var| var.get_var_id() == var_id)
    }

    pub fn get_vars(&self) -> &[AWPacketVar] {
        &self.vars
    }

    pub fn add_byte(&mut self, id: VarID, value: u8) {
        self.add_var(AWPacketVar::Byte(id, value));
    }

    pub fn get_byte(&self, var_id: VarID) -> Option<u8> {
        for var in &self.vars {
            match var {
                AWPacketVar::Byte(id, x) if *id == var_id => return Some(*x),
                _ => {}
            }
        }

        None
    }

    pub fn add_int(&mut self, id: VarID, value: i32) {
        self.add_var(AWPacketVar::Int(id, value));
    }

    pub fn get_int(&self, var_id: VarID) -> Option<i32> {
        for var in &self.vars {
            match var {
                AWPacketVar::Int(id, x) if *id == var_id => return Some(*x),
                _ => {}
            }
        }

        None
    }

    pub fn add_uint(&mut self, id: VarID, value: u32) {
        self.add_var(AWPacketVar::Uint(id, value));
    }

    // Convenience conversion to u32
    pub fn get_uint(&self, var_id: VarID) -> Option<u32> {
        for var in &self.vars {
            match var {
                AWPacketVar::Int(id, x) if *id == var_id => return Some(*x as u32),
                _ => {}
            }
        }

        None
    }

    pub fn add_float(&mut self, id: VarID, value: f32) {
        self.add_var(AWPacketVar::Float(id, value));
    }

    pub fn get_float(&self, var_id: VarID) -> Option<f32> {
        for var in &self.vars {
            match var {
                AWPacketVar::Float(id, x) if *id == var_id => return Some(*x),
                _ => {}
            }
        }

        None
    }

    pub fn add_string(&mut self, id: VarID, value: String) {
        self.add_var(AWPacketVar::String(id, value));
    }

    pub fn get_string(&self, var_id: VarID) -> Option<String> {
        for var in &self.vars {
            match var {
                AWPacketVar::String(id, x) if *id == var_id => return Some(x.clone()),
                _ => {}
            }
        }

        None
    }

    pub fn add_data(&mut self, id: VarID, value: Vec<u8>) {
        self.add_var(AWPacketVar::Data(id, value));
    }

    pub fn get_data(&self, var_id: VarID) -> Option<Vec<u8>> {
        for var in &self.vars {
            match var {
                AWPacketVar::Data(id, x) if *id == var_id => return Some(x.clone()),
                _ => {}
            }
        }

        None
    }

    /// The expected length of the packet after serialization.
    fn serialize_len(&self) -> Result<usize, String> {
        let mut size = TagHeader::length();

        for var in &self.vars {
            let var_serialized_len = var.serialize_len().ok_or(
                "serialize_len calculation failed because a var was too large".to_string(),
            )?;

            size = size.checked_add(var_serialized_len).ok_or(
                "serialize_len calculation failed because the result would have been too large"
                    .to_string(),
            )?;
        }

        Ok(size)
    }

    /// Encode the given packet.
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let serialize_len = self.serialize_len()?;

        if serialize_len > u16::MAX.into() {
            return Err(format!("Serializing packet too large: {serialize_len}"));
        }

        let mut result = Vec::<u8>::with_capacity(serialize_len);
        let serialize_len = serialize_len as u16;

        let header = TagHeader {
            serialized_length: serialize_len,
            header_0: self.header_0,
            opcode: self.opcode.into(),
            header_1: self.header_1,
            var_count: self.vars.len() as u16,
        };

        result.extend(header.serialize());
        for var in &self.vars {
            result.extend(var.serialize()?);
        }

        Ok(result)
    }

    /// Compress data of one or more packets if large enough
    pub fn compress_if_needed(serialized_bytes: &[u8]) -> Result<Vec<u8>, String> {
        if serialized_bytes.len() > 160 {
            // Serialize the packet and compress it
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());

            encoder
                .write_all(serialized_bytes)
                .map_err(|_| "Failed to write to zlib encoder".to_string())?;

            let compressed_bytes = encoder
                .finish()
                .map_err(|_| "Failed to compress".to_string())?;

            let serialized_length_usize =
                compressed_bytes
                    .len()
                    .checked_add(TagHeader::length())
                    .ok_or_else(|| "Serialized length became too large".to_string())?;

            let serialized_length_u16 = u16::try_from(serialized_length_usize)
                .map_err(|_| "Serialized length became too large to fit in a u16".to_string())?;

            // Add a new uncompressed header to the beginning
            let new_header = TagHeader {
                serialized_length: serialized_length_u16,
                header_0: 0,
                opcode: -1,
                header_1: 1, // if self.header_1 != 0 { self.header_1 } else { 1 },
                var_count: 0,
            };

            let mut result = new_header.serialize();
            result.extend(compressed_bytes);

            return Ok(result);
        }

        Ok(serialized_bytes.to_vec())
    }

    /// Decompress a compressed packet and return its decompressed serialized bytes.
    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, String> {
        let compressed_data = data
            .get(TagHeader::length()..)
            .ok_or("Data not long enough to do any decompression".to_string())?;

        let mut decoder = ZlibDecoder::new(compressed_data);
        let mut decompressed_bytes = Vec::<u8>::new();
        match decoder.read_to_end(&mut decompressed_bytes) {
            Ok(_) => Ok(decompressed_bytes),
            Err(_) => Err("Failed to decode compressed data".to_string()),
        }
    }

    /// Decode a packet and return an instance if successful.
    pub fn deserialize(mut data: &[u8]) -> Result<(Self, usize), String> {
        let mut total_consumed: usize = 0;
        let (header, consumed) = TagHeader::deserialize(data)?;
        let consumed: usize = consumed.try_into().map_err(|why| {
            format!("TagHeader::deserialize consumed too many bytes: {consumed} - {why:?}")
        })?;

        data = data
            .get(consumed..)
            .ok_or("Not enough data to deserialize".to_string())?;

        total_consumed = total_consumed
            .checked_add(consumed)
            .ok_or("Consumed too much data while deserializing".to_string())?;

        let mut vars = Vec::<AWPacketVar>::with_capacity(header.var_count as usize);

        for _ in 0..header.var_count {
            let (var, consumed) = AWPacketVar::deserialize(data)?;
            let consumed: usize = consumed.try_into().map_err(|why| {
                format!("AWPacketVar::deserialize consumed too many bytes: {consumed} - {why:?}")
            })?;
            data = data
                .get(consumed..)
                .ok_or("Not enough data to deserialize".to_string())?;

            total_consumed = total_consumed
                .checked_add(consumed)
                .ok_or("Consumed too much data while deserializing".to_string())?;

            vars.push(var);
        }

        if total_consumed != header.serialized_length.into() {
            return Err(format!(
                "Consumed {total_consumed} bytes instead of {}",
                header.serialized_length
            ));
        }

        let opcode = match PacketType::from_i16(header.opcode) {
            Some(packet_type) => PacketTypeResult::PacketType(packet_type),
            None => PacketTypeResult::Unknown(header.opcode),
        };

        Ok((
            Self {
                vars,
                opcode,
                header_0: header.header_0,
                header_1: header.header_1,
            },
            total_consumed,
        ))
    }

    /// Examine serialized header to see what the state of this packet is.
    pub fn deserialize_check(src: &[u8]) -> Result<usize, DeserializeError> {
        let (header, _) = TagHeader::deserialize(src).map_err(|_| DeserializeError::Length)?;

        if !header.is_valid() {
            return Err(DeserializeError::InvalidHeader);
        }

        let serialized_length_usize: usize = header.serialized_length.into();

        if serialized_length_usize > src.len() {
            // We have the header, but we haven't received the rest of the packet yet.
            // Probably need to wait for the next TCP frame.
            return Err(DeserializeError::Length);
        }

        if header.opcode == -1 && header.header_1 != 0 {
            return Err(DeserializeError::Compressed(serialized_length_usize));
        }

        Ok(serialized_length_usize)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AWPacketGroup {
    pub packets: Vec<AWPacket>,
}

impl AWPacketGroup {
    pub fn new() -> Self {
        Self {
            packets: Vec::new(),
        }
    }
    pub fn push(&mut self, packet: AWPacket) -> Result<usize, AWPacket> {
        let packet_serialized_len = match packet.serialize_len() {
            Ok(len) => len,
            Err(_) => return Err(packet),
        };

        let self_serialized_len = match self.serialize_len() {
            Ok(len) => len,
            Err(_) => return Err(packet),
        };

        let total_len = match self_serialized_len.checked_add(packet_serialized_len) {
            Some(len) => len,
            None => return Err(packet),
        };

        if total_len >= 0x8000 {
            return Err(packet);
        }

        self.packets.push(packet);
        Ok(total_len)
    }

    pub fn serialize_len(&self) -> Result<usize, String> {
        let mut total = 0usize;
        for p in &self.packets {
            let packet_serialized_len = p.serialize_len()?;

            total = match total.checked_add(packet_serialized_len) {
                Some(len) => len,
                None => return Err(
                    "serialize_len calculation failed because the result would have been too large"
                        .to_string(),
                ),
            };
        }

        Ok(total)
    }
}

impl Default for AWPacketGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct TagHeader {
    /// The length of the packet
    pub serialized_length: u16,
    /// Purpose not known
    pub header_0: u16,
    /// Packet type
    pub opcode: i16,
    /// Purpose not known
    pub header_1: u16,
    /// Number of variables in this packet
    pub var_count: u16,
}

impl TagHeader {
    #[inline]
    pub fn length() -> usize {
        10
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity(10);
        result
            .write_u16::<BigEndian>(self.serialized_length)
            .unwrap();
        result.write_u16::<BigEndian>(self.header_0).unwrap();
        result.write_i16::<BigEndian>(self.opcode).unwrap();
        result.write_u16::<BigEndian>(self.header_1).unwrap();
        result.write_u16::<BigEndian>(self.var_count).unwrap();

        // This is important because it is going over the network
        assert!(result.len() == TagHeader::length());

        result
    }

    pub fn deserialize(data: &[u8]) -> Result<(Self, u64), String> {
        if data.len() < TagHeader::length() {
            return Err("Not enough data to deserialize TagHeader.".to_string());
        }

        let mut reader = Cursor::new(data);

        let serialized_length = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not read serialized_length.")?;
        let header_0 = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not read header_0.")?;
        let opcode = reader
            .read_i16::<BigEndian>()
            .map_err(|_| "Could not read opcode.")?;
        let header_1 = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not read header_1.")?;
        let var_count = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not read var_count.")?;

        Ok((
            Self {
                serialized_length,
                header_0,
                opcode,
                header_1,
                var_count,
            },
            reader.position(),
        ))
    }

    pub fn is_valid(&self) -> bool {
        if self.header_1 <= 3 || self.opcode == PacketType::Tunnel as i16 {
            if self.var_count > 1024 {
                return false;
            } else {
                if self.header_1 == 0 {
                    return self.opcode == (PacketType::Tunnel as i16);
                }
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
pub enum DeserializeError {
    Length,
    InvalidHeader,
    Compressed(usize),
}

#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum PacketType {
    PublicKeyResponse = 1,
    StreamKeyResponse = 2,

    Address = 5,
    Attributes = 6,
    AttributeChange = 7,
    AttributesReset = 8,
    AvatarAdd = 9,
    AvatarChange = 10,
    AvatarClick = 11,
    AvatarDelete = 12,

    Botgram = 14,
    BotgramResponse = 15,

    Capabilities = 16,
    CellBegin = 17,
    CellEnd = 18,
    CellNext = 19,
    CellUpdate = 20,
    CitizenAdd = 21,
    CitizenInfo = 22,
    CitizenLookupByName = 23,
    CitizenLookupByNumber = 24,
    CitizenChange = 25,
    CitizenDelete = 26,
    CitizenNext = 27,
    CitizenPrev = 28,
    CitizenChangeResult = 29,
    ConsoleMessage = 30,
    ContactAdd = 31,
    ContactChange = 32,
    ContactDelete = 33,
    ContactList = 34,
    Enter = 35,

    PublicKeyRequest = 36,
    Heartbeat = 37,
    Identify = 38,
    LicenseAdd = 39,
    LicenseResult = 40,
    LicenseByName = 41,
    LicenseChange = 42,
    LicenseDelete = 43,
    LicenseNext = 44,
    LicensePrev = 45,
    LicenseChangeResult = 46,
    Login = 47,
    Message = 48,
    ObjectAdd = 49,

    ObjectClick = 51,
    ObjectDelete = 52,
    ObjectDeleteAll = 53,

    ObjectResult = 55,
    ObjectSelect = 56,

    QueryNeedMore = 59,
    QueryUpToDate = 60,
    RegistryReload = 61,
    ServerLogin = 62,
    WorldServerStart = 63,

    ServerWorldDelete = 67,
    ServerWorldList = 68,
    ServerWorldListResult = 69,
    ServerWorldResult = 70,

    TelegramDeliver = 75,
    TelegramGet = 76,
    TelegramNotify = 77,
    TelegramSend = 78,
    Teleport = 79,
    TerrainBegin = 80,
    TerrainChanged = 81,
    TerrainData = 82,
    TerrainDelete = 83,
    TerrainEnd = 84,
    TerrainLoad = 85,
    TerrainNext = 86,

    TerrainSet = 88,
    ToolbarClick = 89,
    URL = 90,
    URLClick = 91,
    UserList = 92,
    UserListResult = 93,
    LoginApplication = 94,

    WorldList = 96,
    WorldListResult = 97,
    WorldLookup = 98,
    WorldStart = 99,
    WorldStop = 100,
    Tunnel = 101,
    WorldStatsUpdate = 102,
    JoinRequest = 103,
    JoinReply = 104,
    Xfer = 105,
    XferReply = 106,
    Noise = 107,

    Camera = 109,
    Botmenu = 110,
    BotmenuResult = 111,
    EjectionInfo = 112,
    EjectAdd = 113,
    EjectDelete = 114,
    EjectLookup = 115,
    EjectNext = 116,
    EjectPrev = 117,
    EjectResult = 118,
    WorldConnectionResult = 119,
    ObjectBump = 120,
    PasswordSend = 121,

    CavTemplateByNumber = 123,
    CavTemplateNext = 124,
    CavTemplateChange = 125,
    CavTemplateDelete = 126,
    WorldCAVDefinitionChange = 127,
    WorldCAV = 128,

    CavDelete = 130,
    WorldCAVResult = 131,
    MoverAdd = 144,
    MoverDelete = 145,
    MoverChange = 146,

    MoverRiderAdd = 148,
    MoverRiderDelete = 149,
    MoverRiderChange = 150,
    MoverLinks = 151,

    SetAFK = 152,

    Immigrate = 155,
    ImmigrateResponse = 156,
    Register = 157,

    AvatarReload = 159,
    WorldInstanceSet = 160,
    WorldInstanceGet = 161,

    ContactConfirm = 163,

    HudCreate = 164,
    HudClick = 165,
    HudDestroy = 166,
    HudClear = 167,
    HudResult = 168,
    AvatarLocation = 169,
    ObjectQuery = 170,
    LaserBeam = 183,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PacketTypeResult {
    PacketType(PacketType),
    Unknown(i16),
}

impl From<PacketTypeResult> for i16 {
    fn from(value: PacketTypeResult) -> Self {
        match value {
            PacketTypeResult::PacketType(packet_type) => packet_type as i16,
            PacketTypeResult::Unknown(num) => num,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_serialize() {
        let mut packet = AWPacket::new(PacketType::Address);
        packet.add_var(AWPacketVar::String(VarID::AFKStatus, "Hello".to_string()));
        packet.add_var(AWPacketVar::Byte(VarID::AttributeAllowTourists, 1));
        let serialized = packet.serialize().unwrap();
        let (deserialized, _) = AWPacket::deserialize(&serialized).unwrap();
        assert!(packet == deserialized);
    }
}
