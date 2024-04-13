use num_derive::FromPrimitive;

#[derive(FromPrimitive, Debug, Copy, Clone, Default, PartialEq)]
pub enum WorldRating {
    #[default]
    G = 0,
    PG = 1,
    PG13 = 2,
    R = 3,
    X = 4,
}

impl WorldRating {
    pub fn from_u8(value: u8) -> Option<Self> {
        num_traits::FromPrimitive::from_u8(value)
    }
}

#[derive(Debug)]
pub struct World {
    pub name: String,
    pub free_entry: bool,
    pub world_size: u32,
    pub max_users: u32,
    pub rating: WorldRating,
    pub user_count: u32,
}

#[derive(Debug)]
pub struct WorldServer {
    pub build: u32,
    pub server_port: u16,
    pub worlds: Vec<World>,
}

impl WorldServer {
    pub fn get_world(&self, name: &str) -> Option<&World> {
        self.worlds
            .iter()
            .find(|&w| w.name.eq_ignore_ascii_case(name))
    }

    pub fn get_world_mut(&mut self, name: &str) -> Option<&mut World> {
        self.worlds
            .iter_mut()
            .find(|w| w.name.eq_ignore_ascii_case(name))
    }

    pub fn remove_world(&mut self, name: &str) -> Option<World> {
        for (i, world) in self.worlds.iter().enumerate() {
            if world.name.eq_ignore_ascii_case(name) {
                return Some(self.worlds.remove(i));
            }
        }
        None
    }
}
