mod contact_list;
mod player_list;
mod world_list;

pub use contact_list::{
    regenerate_contact_list, regenerate_contact_list_and_mutuals, ContactList, ContactListEntry,
    ContactState,
};
pub use player_list::{regenerate_player_list, PlayerListEntry};
pub use world_list::{regenerate_world_list, WorldListEntry, WorldStatus};

#[derive(Debug)]
pub struct Tabs {
    pub contact_list: contact_list::UpdatingContactList,
    pub player_list: player_list::UpdatingPlayerList,
    pub world_list: world_list::UpdatingWorldList,
}

impl Tabs {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self {
            contact_list: contact_list::UpdatingContactList::new(),
            player_list: player_list::UpdatingPlayerList::new(),
            world_list: world_list::UpdatingWorldList::new(),
        }
    }
}
