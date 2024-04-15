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

use crate::{database::citizen::CitizenQuery, UniverseConnection};
use aw_core::*;
use aw_db::DatabaseResult;

/// Helper function for all the packets involved in the citizen lookup admin menu
fn try_citizen_lookup(
    conn: &UniverseConnection,
    response: &mut AWPacket,
    how: impl Fn() -> DatabaseResult<Option<CitizenQuery>>,
) -> ReasonCode {
    if !conn.has_admin_permissions() {
        return ReasonCode::Unauthorized;
    }

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
        AWPacketVar::Uint(VarID::CitizenNumber, citizen.id),
        AWPacketVar::String(VarID::CitizenName, citizen.name.clone()),
        AWPacketVar::String(VarID::CitizenURL, citizen.url.clone()),
        AWPacketVar::Byte(VarID::TrialUser, citizen.trial as u8),
        AWPacketVar::Byte(VarID::CAVEnabled, citizen.cav_enabled as u8),
        AWPacketVar::Uint(
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
            AWPacketVar::Uint(VarID::CitizenImmigration, citizen.immigration),
            AWPacketVar::Uint(VarID::CitizenExpiration, citizen.expiration),
            AWPacketVar::Uint(VarID::CitizenLastLogin, citizen.last_login),
            AWPacketVar::Uint(VarID::CitizenTotalTime, citizen.total_time),
            AWPacketVar::Uint(VarID::CitizenBotLimit, citizen.bot_limit),
            AWPacketVar::Byte(VarID::BetaUser, citizen.beta as u8),
            AWPacketVar::Byte(VarID::CitizenEnabled, citizen.enabled as u8),
            AWPacketVar::Uint(VarID::CitizenPrivacy, citizen.privacy),
            AWPacketVar::String(VarID::CitizenPassword, citizen.password.clone()),
            AWPacketVar::String(VarID::CitizenEmail, citizen.email.clone()),
            AWPacketVar::String(VarID::CitizenPrivilegePassword, citizen.priv_pass.clone()),
            AWPacketVar::Uint(VarID::CitizenImmigration, citizen.immigration),
        ]);
    }

    if admin_vars {
        vars.extend(vec![
            AWPacketVar::String(VarID::CitizenComment, citizen.comment.clone()),
            AWPacketVar::Uint(VarID::IdentifyUserIP, citizen.last_address),
        ]);
    }

    vars
}
