mod login;
pub use login::*;

mod citizen;
pub use citizen::*;

mod license;
pub use license::*;

mod contact;
pub use contact::*;

mod telegram_send;
pub use telegram_send::telegram_send;

mod telegram_get;
pub use telegram_get::telegram_get;

mod attribute_change;
pub use attribute_change::attribute_change;

mod world_list;
pub use world_list::world_list;

mod world_lookup;
pub use world_lookup::world_lookup;

mod join;
pub use join::*;

mod botgram;
pub use botgram::botgram;

mod immigrate;
pub use immigrate::immigrate;

mod heartbeat;
pub use heartbeat::heartbeat;

mod user_list;
pub use user_list::user_list;

mod eject;
pub use eject::*;

mod cav_get;
pub use cav_get::get_cav;

use aw_core::*;

fn check_valid_name(mut name: &str, is_tourist: bool) -> Result<(), ReasonCode> {
    if is_tourist {
        // Tourist names must start and end with quotes
        if !name.starts_with('"') || !name.ends_with('"') {
            return Err(ReasonCode::NoSuchCitizen);
        }

        // Strip quotes to continue check
        let name_start = 1;
        let name_end = name.len().checked_sub(1).ok_or(ReasonCode::NameTooShort)?;
        name = name
            .get(name_start..name_end)
            .ok_or(ReasonCode::NameTooShort)?;
    }

    if name.len() < 2 {
        return Err(ReasonCode::NameTooShort);
    }

    if name.ends_with(' ') {
        return Err(ReasonCode::NameEndsWithBlank);
    }

    if name.starts_with(' ') {
        return Err(ReasonCode::NameContainsInvalidBlank);
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == ' ') {
        return Err(ReasonCode::NameContainsNonalphanumericChar);
    }

    Ok(())
}
