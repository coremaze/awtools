#[derive(Debug)]
pub struct PlayerInfo {
    pub build: i32,
    pub session_id: u16,
    pub citizen_id: Option<u32>,
    pub privilege_id: Option<u32>,
    pub username: String,
    pub nonce: Option<[u8; 255]>, // AW4 worlds allow 256 bytes, AW5 worlds allow 255 bytes
    pub world: Option<String>,
}

impl PlayerInfo {
    pub fn effective_privilege(&self) -> u32 {
        match self.privilege_id {
            Some(id) => match id {
                0 => self.citizen_id.unwrap_or(0),
                _ => id,
            },
            None => self.citizen_id.unwrap_or(0),
        }
    }
}
