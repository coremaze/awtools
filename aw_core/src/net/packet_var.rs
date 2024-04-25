//! Packet variable (de)serialization for AW

use crate::encoding::{latin1_to_string, string_to_latin1};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Read, Write};

#[derive(FromPrimitive)]
enum DataType {
    Unknown = 0,
    Byte = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Data = 5,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PacketData {
    Unknown(Vec<u8>),
    Byte(u8),
    Int(i32),
    Uint(u32),
    Float(f32),
    String(String),
    Data(Vec<u8>),
}

impl PacketData {
    fn get_data_type(&self) -> DataType {
        match self {
            Self::Byte(_) => DataType::Byte,
            Self::Int(_) => DataType::Int,
            // Uint being DataType::Int is intentional. This does not have its
            // own ID, it is only for convenience.
            Self::Uint(_) => DataType::Int,
            Self::Float(_) => DataType::Float,
            Self::String(_) => DataType::String,
            Self::Data(_) => DataType::Data,
            Self::Unknown(_) => DataType::Unknown,
        }
    }

    fn get_data_size(&self) -> Option<usize> {
        Some(match self {
            Self::Byte(_) => 1,
            Self::Int(_) => 4,
            Self::Uint(_) => 4,
            Self::Float(_) => 4,
            Self::String(string) => string_to_latin1(string).len().checked_add(1)?,
            Self::Data(buf) => buf.len(),
            Self::Unknown(buf) => buf.len(),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AWPacketVar {
    pub id: u16,
    pub data: PacketData,
}

#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
pub enum VarID {
    VolumeSerial = 6,

    IdentifyUserIP = 26,

    PositionNorth = 36,
    PositionHeight = 37,
    PositionRotation = 38,
    PositionWest = 39,

    BotgramFromCitizenNumber = 40,
    BotgramFromUsername = 41,
    BotgramMessage = 42,
    BotgramCitizenNumber = 43,
    BotgramType = 44,

    BetaUser = 50,
    CitizenBotLimit = 51,
    CitizenComment = 52,
    CitizenEmail = 53,
    CitizenEnabled = 54,
    CitizenExpiration = 55,
    CitizenImmigration = 56,
    CitizenLastLogin = 57,
    CitizenName = 58,
    CitizenNumber = 59,
    CitizenPassword = 60,
    CitizenPrivilegePassword = 61,
    CitizenRemainingTimeUntilExpiration = 62,
    CitizenTotalTime = 63,
    CitizenURL = 64,
    UserType = 67,
    BrowserBuild = 68,
    ContactListMore = 74,
    ContactListName = 75,
    ContactListCitizenID = 76,
    ContactListOptions = 77,
    ContactListStatus = 78,
    ContactListWorld = 79,
    EncryptionKey = 80,
    WorldLicenseComment = 81,
    WorldLicenseCreation = 82,
    WorldLicenseEmail = 83,
    WorldLicenseExpiration = 84,
    WorldLicenseHidden = 85,
    WorldLicenseLastAddress = 86,
    WorldLicenseLastStart = 87,
    WorldName = 88,
    WorldLicenseID = 89,
    WorldLicensePassword = 90,
    WorldLicenseRange = 91,
    WorldLicenseTourists = 92,
    WorldLicenseUsers = 93,
    Application = 94,
    Email = 95,
    LoginUsername = 96,
    LoginID = 97,
    Password = 98,
    PrivilegeUsername = 99,
    PrivilegeUserID = 100,
    PrivilegePassword = 101,
    PlayerPort = 120,
    ReasonCode = 121,
    SessionID = 140,
    TelegramCitizenName = 141,
    TelegramMessage = 142,
    TelegramsMoreRemain = 143,
    TelegramAge = 144,
    TelegramTo = 145,
    UniverseLicense = 171,
    UserListAddress = 176,
    UserListCitizenID = 177,
    UserListEmailAddress = 178,
    UserListID = 180,
    UserListMore = 181,
    UserListName = 182,
    UserListPrivilegeID = 183,
    UserListContinuationID = 184,
    UserListState = 185,
    UserListWorldName = 186,
    WorldFreeEntry = 187,
    WorldAddress = 188,
    WorldBuild = 189,
    WorldUserNonce = 190,
    WorldPort = 191,
    WorldRating = 192,
    WorldListMore = 193,
    WorldListName = 194,
    WorldListRating = 195,
    WorldList3DayUnknown = 196,
    WorldListStatus = 197,
    WorldListUsers = 198,
    WorldUsers = 201,
    BrowserVersion = 211,
    EjectionAddress = 216,
    EjectionCreation = 217,
    EjectionExpiration = 218,
    EjectionComment = 219,
    CAVEnabled = 226,
    CAVTemplate = 227,
    AFKStatus = 261,
    WorldLicenseVoip = 263,
    WorldLicensePlugins = 264,
    CitizenPrivacy = 301,
    TrialUser = 302,
}

impl From<VarID> for u16 {
    fn from(value: VarID) -> Self {
        value as u16
    }
}

impl AWPacketVar {
    pub fn new(var_id: impl Into<u16>, packet_data: PacketData) -> Self {
        Self {
            id: var_id.into(),
            data: packet_data,
        }
    }

    pub fn unknown(var_id: impl Into<u16>, data: Vec<u8>) -> Self {
        Self::new(var_id, PacketData::Unknown(data))
    }

    pub fn byte(var_id: impl Into<u16>, data: u8) -> Self {
        Self::new(var_id, PacketData::Byte(data))
    }

    pub fn int(var_id: impl Into<u16>, data: i32) -> Self {
        Self::new(var_id, PacketData::Int(data))
    }

    pub fn uint(var_id: impl Into<u16>, data: u32) -> Self {
        Self::new(var_id, PacketData::Uint(data))
    }

    pub fn float(var_id: impl Into<u16>, data: f32) -> Self {
        Self::new(var_id, PacketData::Float(data))
    }

    pub fn string(var_id: impl Into<u16>, data: String) -> Self {
        Self::new(var_id, PacketData::String(data))
    }

    pub fn data(var_id: impl Into<u16>, data: Vec<u8>) -> Self {
        Self::new(var_id, PacketData::Data(data))
    }

    pub fn get_var_id(&self) -> u16 {
        self.id
    }

    fn get_data_type(&self) -> DataType {
        self.data.get_data_type()
    }

    fn get_data_size(&self) -> Option<usize> {
        self.data.get_data_size()
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::<u8>::with_capacity(16);

        let var_id = self.get_var_id();

        let size = self
            .get_data_size()
            .ok_or("Data size invalid".to_string())?;

        if size > 0xFFF {
            return Err(format!("Data size is too large: {size}"));
        }
        let size: u16 = size as u16;

        let data_type = self.get_data_type() as u16;

        // Assemble data

        // This header data is big endian
        result.write_u16::<BigEndian>(var_id).unwrap();
        result
            .write_u16::<BigEndian>(data_type << 12 | size)
            .unwrap();

        // Little endian time 😎
        match &self.data {
            PacketData::Byte(x) => {
                result.write_u8(*x).unwrap();
            }
            PacketData::Int(x) => {
                result.write_i32::<LittleEndian>(*x).unwrap();
            }
            PacketData::Uint(x) => {
                result.write_u32::<LittleEndian>(*x).unwrap();
            }
            PacketData::Float(x) => {
                result.write_f32::<LittleEndian>(*x).unwrap();
            }
            PacketData::String(x) => {
                result.write_all(&string_to_latin1(x)).unwrap();
                result.write_all(&[0u8]).unwrap();
            }
            PacketData::Data(x) => {
                result.write_all(x).unwrap();
            }
            PacketData::Unknown(x) => {
                result.write_all(x).unwrap();
            }
        };

        Ok(result)
    }

    pub fn deserialize(data: &[u8]) -> Result<(Self, u64), String> {
        let mut reader = Cursor::new(data);

        // Header is big endian
        let var_id_num = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not deserialize var_id_num")?;
        let data_type_and_size = reader
            .read_u16::<BigEndian>()
            .map_err(|_| "Could not deserialize data_type_and_size")?;

        // Extract size and data type from packed value
        let size = data_type_and_size & 0xFFF;
        let data_type_num = (data_type_and_size & 0xF000) >> 12;

        let data_type: DataType = DataType::from_u16(data_type_num)
            .ok_or_else(|| format!("Received invalid data type {data_type_num}"))?;

        // Little endian
        let result = match data_type {
            DataType::Byte => {
                let x = reader
                    .read_u8()
                    .map_err(|_| "Could not deserialize Byte data")?;
                Self::byte(var_id_num, x)
            }
            DataType::Int => {
                let x = reader
                    .read_i32::<LittleEndian>()
                    .map_err(|_| "Could not deserialize Int data")?;
                Self::int(var_id_num, x)
            }
            DataType::Float => {
                let x = reader
                    .read_f32::<LittleEndian>()
                    .map_err(|_| "Could not deserialize Float data")?;
                Self::float(var_id_num, x)
            }
            DataType::String => {
                let mut buf = vec![0u8; size as usize];
                reader
                    .read_exact(&mut buf)
                    .map_err(|_| "Could not deserialize String data")?;
                Self::string(var_id_num, latin1_to_string(&buf))
            }
            DataType::Data => {
                let mut buf = vec![0u8; size as usize];
                reader
                    .read_exact(&mut buf)
                    .map_err(|_| "Could not deserialize Data data")?;
                Self::data(var_id_num, buf)
            }
            DataType::Unknown => {
                let mut buf = vec![0u8; size as usize];
                reader
                    .read_exact(&mut buf)
                    .map_err(|_| "Could not deserialize Unknown data")?;
                AWPacketVar::unknown(var_id_num, buf)
            }
        };

        Ok((result, reader.position()))
    }

    pub fn serialize_len(&self) -> Option<usize> {
        let var_id_size: usize = 2;
        let data_type_and_size_size: usize = 2;
        let data_size: usize = self.get_data_size()?;

        var_id_size
            .checked_add(data_type_and_size_size)?
            .checked_add(data_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_byte() {
        let var = AWPacketVar::byte(1u16, 123u8);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len().unwrap() == data.len());
    }

    #[test]
    pub fn test_int() {
        let var = AWPacketVar::int(1u16, 0x12345678);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len().unwrap() == data.len());
    }

    #[test]
    pub fn test_float() {
        let var = AWPacketVar::float(1u16, 3.141_592_7);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len().unwrap() == data.len());
    }

    #[test]
    pub fn test_string() {
        let var = AWPacketVar::string(1u16, "Hello, World!".to_string());
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len().unwrap() == data.len());
    }

    #[test]
    pub fn test_data() {
        let var = AWPacketVar::data(1u16, vec![0u8, 1, 3, 5, 7, 8, 4, 2, 5, 23, 111, 222]);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len().unwrap() == data.len());
    }
}
