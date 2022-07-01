//! Packet variable (de)serialization for AW

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Read, Write};

#[derive(FromPrimitive)]
pub enum DataType {
    Byte = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Data = 5,
}

#[derive(Debug, PartialEq)]
pub enum AWPacketVar {
    Byte(VarID, u8),
    Int(VarID, i32),
    Float(VarID, f32),
    String(VarID, String),
    Data(VarID, Vec<u8>),
}

#[derive(FromPrimitive, Clone, Copy, Debug, PartialEq)]
pub enum VarID {
    // These have the same IDs as the attributes,
    // but are for packets
    Attrib_AllowTourists = 0,
    Attrib_UnknownBilling1 = 1,
    Attrib_BetaBrowser = 2,
    Attrib_MinimumBrowser = 3,
    Attrib_LatestBrowser = 4,
    Attrib_UniverseBuild = 5,
    Attrib_CitizenChanges = 6,
    Attrib_UnknownBilling7 = 7,
    Attrib_BillingMethod = 8,
    Attrib_BillingUnknown9 = 9,
    Attrib_SearchTabURL = 10,
    Attrib_Timestamp = 11,
    Attrib_WelcomeMessage = 12,
    Attrib_BetaWorld = 13,
    Attrib_MinimumWorld = 14,
    Attrib_LatestWorld = 15,
    Attrib_DefaultStartWorld = 16,
    Attrib_Userlist = 17,
    Attrib_NotepadTabURL = 18,
    Attrib_MailTemplate = 19,
    Attrib_MailFile = 20,
    Attrib_MailCommand = 21,
    Attrib_PAVObjectPath = 22,
    Attrib_UnknownUniverseSetting = 23,

    Identify_UserIP = 26,

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
    WorldStart_WorldName = 88,
    WorldLicenseID = 89,
    WorldLicensePassword = 90,
    WorldLicenseRange = 91,
    WorldLicenseTourists = 92,
    WorldLicenseUsers = 93,
    Email = 95,
    LoginUsername = 96,
    LoginID = 97,
    Password = 98,
    PrivilegeUsername = 99,
    PrivilegeUserID = 100,
    PrivilegePassword = 101,
    ReasonCode = 121,
    SessionID = 140,
    TelegramCitizenName = 141,
    TelegramMessage = 142,
    TelegramsMoreRemain = 143,
    TelegramAge = 144,
    TelegramTo = 145,
    UniverseLicense = 171,
    UserList_Address = 176,
    UserList_CitizenID = 177,
    UserList_ID = 180,
    UserList_More = 181,
    UserList_Name = 182,
    UserList_PrivilegeID = 183,
    UserList_State = 185,
    UserList_WorldName = 186,
    WorldFreeEntry = 187,
    WorldAddress = 188,
    WorldBuild = 189,
    WorldUserNonce = 190,
    WorldPort = 191,
    WorldRating = 192,
    WorldList_More = 193,
    WorldList_Name = 194,
    WorldList_Rating = 195,
    WorldList_Status = 197,
    WorldList_Users = 198,
    World_Users = 201,
    BrowserVersion = 211,
    CAVEnabled = 226,
    CAVTemplate = 227,
    AFKStatus = 261,
    WorldLicenseVoip = 263,
    WorldLicensePlugins = 264,
    CitizenPrivacy = 301,
    TrialUser = 302,

    Unknown = 65535,
}

fn latin1_to_string(s: &[u8]) -> String {
    s.iter()
        .map(|&c| c as char)
        .collect::<String>()
        .trim_end_matches('\0') // Strip off any null terminator
        .to_string()
}

fn string_to_latin1(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u8).collect()
}

impl AWPacketVar {
    pub fn get_var_id(&self) -> VarID {
        match &self {
            AWPacketVar::Byte(var_id, _) => *var_id,
            AWPacketVar::Int(var_id, _) => *var_id,
            AWPacketVar::Float(var_id, _) => *var_id,
            AWPacketVar::String(var_id, _) => *var_id,
            AWPacketVar::Data(var_id, _) => *var_id,
        }
    }

    pub fn get_data_type(&self) -> DataType {
        match self {
            AWPacketVar::Byte(_, _) => DataType::Byte,
            AWPacketVar::Int(_, _) => DataType::Int,
            AWPacketVar::Float(_, _) => DataType::Float,
            AWPacketVar::String(_, _) => DataType::String,
            AWPacketVar::Data(_, _) => DataType::Data,
        }
    }

    fn get_data_size(&self) -> usize {
        match self {
            AWPacketVar::Byte(_, _) => 1,
            AWPacketVar::Int(_, _) => 4,
            AWPacketVar::Float(_, _) => 4,
            AWPacketVar::String(_, string) => string_to_latin1(string).len() + 1,
            AWPacketVar::Data(_, buf) => buf.len(),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::<u8>::with_capacity(16);

        let var_id = self.get_var_id() as u16;
        let size: usize = self.get_data_size();
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
        match &self {
            AWPacketVar::Byte(_, x) => {
                result.write_u8(*x).unwrap();
            }
            AWPacketVar::Int(_, x) => {
                result.write_i32::<LittleEndian>(*x).unwrap();
            }
            AWPacketVar::Float(_, x) => {
                result.write_f32::<LittleEndian>(*x).unwrap();
            }
            AWPacketVar::String(_, x) => {
                result.write_all(&string_to_latin1(x)).unwrap();
                result.write_all(&[0u8]).unwrap();
            }
            AWPacketVar::Data(_, x) => {
                result.write_all(x).unwrap();
            }
        };

        Ok(result)
    }

    pub fn deserialize(data: &[u8]) -> Result<(Self, usize), String> {
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

        let var_id: VarID = VarID::from_u16(var_id_num).unwrap_or_else(|| {
            eprintln!("Received unknown variable id {var_id_num}");
            VarID::Unknown
        });

        let data_type: DataType = DataType::from_u16(data_type_num)
            .ok_or_else(|| format!("Received invalid data type {data_type_num}"))?;

        // Little endian
        let result = match data_type {
            DataType::Byte => {
                let x = reader
                    .read_u8()
                    .map_err(|_| "Could not deserialize Byte data")?;
                AWPacketVar::Byte(var_id, x)
            }
            DataType::Int => {
                let x = reader
                    .read_i32::<LittleEndian>()
                    .map_err(|_| "Could not deserialize Int data")?;
                AWPacketVar::Int(var_id, x)
            }
            DataType::Float => {
                let x = reader
                    .read_f32::<LittleEndian>()
                    .map_err(|_| "Could not deserialize Float data")?;
                AWPacketVar::Float(var_id, x)
            }
            DataType::String => {
                let mut buf = vec![0u8; size as usize];
                reader
                    .read_exact(&mut buf)
                    .map_err(|_| "Could not deserialize String data")?;
                AWPacketVar::String(var_id, latin1_to_string(&buf))
            }
            DataType::Data => {
                let mut buf = vec![0u8; size as usize];
                reader
                    .read_exact(&mut buf)
                    .map_err(|_| "Could not deserialize Data data")?;
                AWPacketVar::Data(var_id, buf)
            }
        };

        Ok((result, reader.position().try_into().unwrap()))
    }

    pub fn serialize_len(&self) -> usize {
        2 /* var id */
        + 2 /* data type and size */
        + self.get_data_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_byte() {
        let var = AWPacketVar::Byte(VarID::AFKStatus, 123u8);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len() == data.len());
    }

    #[test]
    pub fn test_int() {
        let var = AWPacketVar::Int(VarID::AFKStatus, 0x12345678);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len() == data.len());
    }

    #[test]
    pub fn test_float() {
        let var = AWPacketVar::Float(VarID::AFKStatus, 3.14159265);
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len() == data.len());
    }

    #[test]
    pub fn test_string() {
        let var = AWPacketVar::String(VarID::AFKStatus, "Hello, World!".to_string());
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len() == data.len());
    }

    #[test]
    pub fn test_data() {
        let var = AWPacketVar::Data(
            VarID::AFKStatus,
            vec![0u8, 1, 3, 5, 7, 8, 4, 2, 5, 23, 111, 222],
        );
        let data = var.serialize().unwrap();
        let (decoded, _) = AWPacketVar::deserialize(&data).unwrap();
        assert!(var == decoded);
        assert!(var.serialize_len() == data.len());
    }
}
