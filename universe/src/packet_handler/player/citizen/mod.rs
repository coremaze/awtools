mod citizen_next;
pub use citizen_next::citizen_next;

mod citizen_prev;
pub use citizen_prev::citizen_prev;

mod citizen_lookup_by_name;
pub use citizen_lookup_by_name::citizen_lookup_by_name;

mod citizen_lookup_by_number;
pub use citizen_lookup_by_number::citizen_lookup_by_number;

mod citizen_change;
pub use citizen_change::citizen_change;

mod citizen_add;
pub use citizen_add::citizen_add;

mod citizen_delete;
pub use citizen_delete::citizen_delete;

use crate::{database::citizen::CitizenQuery, UniverseConnection};
use aw_core::*;
use aw_db::DatabaseResult;

/// Helper function for all the packets involved in the citizen lookup admin menu
fn try_citizen_lookup(
    conn: &UniverseConnection,
    response: &mut AWPacket,
    how: impl Fn() -> DatabaseResult<Option<CitizenQuery>>,
) -> ReasonCode {
    let Some(player_citizen) = conn.client.as_ref().and_then(|x| x.citizen()) else {
        // The user attempting to do this should be a citizen
        return ReasonCode::Unauthorized;
    };

    match how() {
        DatabaseResult::Ok(Some(citizen)) => {
            let same_citizen_id = citizen.id == player_citizen.cit_id;
            let is_admin = conn.has_admin_permissions();
            let vars = citizen_info_vars(&citizen, same_citizen_id, is_admin);
            for v in vars {
                response.add_var(v);
            }

            ReasonCode::Success
        }
        DatabaseResult::Ok(None) => ReasonCode::NoSuchCitizen,
        DatabaseResult::DatabaseError => ReasonCode::DatabaseError,
    }
}

fn citizen_info_vars(
    citizen: &CitizenQuery,
    self_vars: bool,
    admin_vars: bool,
) -> Vec<AWPacketVar> {
    let mut vars = vec![
        AWPacketVar::uint(VarID::CitizenNumber, citizen.id),
        AWPacketVar::string(VarID::CitizenName, citizen.name.clone()),
        AWPacketVar::string(VarID::CitizenURL, citizen.url.clone()),
        AWPacketVar::byte(VarID::TrialUser, citizen.trial as u8),
        AWPacketVar::byte(VarID::CAVEnabled, citizen.cav_enabled as u8),
        AWPacketVar::uint(
            VarID::CAVTemplate,
            if citizen.cav_enabled != 0 {
                citizen.cav_template
            } else {
                0
            },
        ),
    ];

    if self_vars || admin_vars {
        vars.extend(vec![
            AWPacketVar::uint(VarID::CitizenImmigration, citizen.immigration),
            AWPacketVar::uint(VarID::CitizenExpiration, citizen.expiration),
            AWPacketVar::uint(VarID::CitizenLastLogin, citizen.last_login),
            AWPacketVar::uint(VarID::CitizenTotalTime, citizen.total_time),
            AWPacketVar::uint(VarID::CitizenBotLimit, citizen.bot_limit),
            AWPacketVar::byte(VarID::BetaUser, citizen.beta as u8),
            AWPacketVar::byte(VarID::CitizenEnabled, citizen.enabled as u8),
            AWPacketVar::uint(VarID::CitizenPrivacy, citizen.privacy),
            AWPacketVar::string(VarID::CitizenPassword, citizen.password.clone()),
            AWPacketVar::string(VarID::CitizenEmail, citizen.email.clone()),
            AWPacketVar::string(VarID::CitizenPrivilegePassword, citizen.priv_pass.clone()),
            AWPacketVar::uint(VarID::CitizenImmigration, citizen.immigration),
        ]);
    }

    if admin_vars {
        vars.extend(vec![
            AWPacketVar::string(VarID::CitizenComment, citizen.comment.clone()),
            AWPacketVar::uint(VarID::IdentifyUserIP, citizen.last_address),
        ]);
    }

    vars
}
