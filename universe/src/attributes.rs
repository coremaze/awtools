use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::attrib::{AttribDB, Attribute};
use crate::database::Database;
use crate::{AWPacket, PacketType, UniverseConnection, VarID};

pub fn send_attributes(conn: &UniverseConnection, database: &Database) {
    let mut packet = AWPacket::new(PacketType::Attributes);
    packet.set_header_0(0);
    packet.set_header_1(0);

    log::trace!("get attributes");
    let attribs = get_attributes(database);
    log::trace!("get attributes done");

    packet.add_string(
        VarID::AttributeAllowTourists,
        attribs
            .get(&Attribute::AllowTourists)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeBetaBrowser,
        attribs
            .get(&Attribute::BetaBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeUniverseBuild,
        attribs
            .get(&Attribute::UniverseBuild)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeCitizenChanges,
        attribs
            .get(&Attribute::CitizenChanges)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeSearchTabURL,
        attribs
            .get(&Attribute::SearchTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeTimestamp,
        attribs
            .get(&Attribute::Timestamp)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeWelcomeMessage,
        attribs
            .get(&Attribute::WelcomeMessage)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeBetaWorld,
        attribs
            .get(&Attribute::BetaWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeMinimumWorld,
        attribs
            .get(&Attribute::MinimumWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeLatestWorld,
        attribs
            .get(&Attribute::LatestWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeDefaultStartWorld,
        attribs
            .get(&Attribute::DefaultStartWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeUserlist,
        attribs
            .get(&Attribute::Userlist)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeNotepadTabURL,
        attribs
            .get(&Attribute::NotepadTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeMinimumBrowser,
        attribs
            .get(&Attribute::MinimumBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeLatestBrowser,
        attribs
            .get(&Attribute::LatestBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        VarID::AttributeUnknownBilling7,
        attribs
            .get(&Attribute::UnknownBilling7)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(VarID::AttributeBillingMethod, "".to_string());
    packet.add_string(
        VarID::AttributeBillingUnknown9,
        attribs
            .get(&Attribute::BillingUnknown9)
            .unwrap_or(&String::new())
            .to_string(),
    );

    conn.send(packet);
}

pub fn get_attributes(database: &Database) -> HashMap<Attribute, String> {
    let mut result = match database.attrib_get() {
        Ok(attribs) => attribs,
        Err(_) => {
            log::warn!("Unable to get universe attributes from database, but we are continuing since we can still provide time and universe build");
            HashMap::<Attribute, String>::new()
        }
    };

    // Not sure if the client actually cares about what build we use here
    result.insert(Attribute::UniverseBuild, "120".to_string());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Current time is before the unix epoch.")
        .as_secs();

    result.insert(Attribute::Timestamp, now.to_string());

    result
}

pub fn set_attribute(var_id: VarID, value: &str, database: &Database) {
    let id = match var_id {
        VarID::AttributeAllowTourists => Attribute::AllowTourists,
        VarID::AttributeUnknownBilling1 => Attribute::UnknownBilling1,
        VarID::AttributeBetaBrowser => Attribute::BetaBrowser,
        VarID::AttributeMinimumBrowser => Attribute::MinimumBrowser,
        VarID::AttributeLatestBrowser => Attribute::LatestBrowser,
        VarID::AttributeUniverseBuild => Attribute::UniverseBuild,
        VarID::AttributeCitizenChanges => Attribute::CitizenChanges,
        VarID::AttributeUnknownBilling7 => Attribute::UnknownBilling7,
        VarID::AttributeBillingMethod => Attribute::BillingMethod,
        VarID::AttributeBillingUnknown9 => Attribute::BillingUnknown9,
        VarID::AttributeSearchTabURL => Attribute::SearchTabURL,
        VarID::AttributeTimestamp => Attribute::Timestamp,
        VarID::AttributeWelcomeMessage => Attribute::WelcomeMessage,
        VarID::AttributeBetaWorld => Attribute::BetaWorld,
        VarID::AttributeMinimumWorld => Attribute::MinimumWorld,
        VarID::AttributeLatestWorld => Attribute::LatestWorld,
        VarID::AttributeDefaultStartWorld => Attribute::DefaultStartWorld,
        VarID::AttributeUserlist => Attribute::Userlist,
        VarID::AttributeNotepadTabURL => Attribute::NotepadTabURL,
        VarID::AttributeMailTemplate => Attribute::MailTemplate,
        VarID::AttributeMailFile => Attribute::MailFile,
        VarID::AttributeMailCommand => Attribute::MailCommand,
        VarID::AttributePAVObjectPath => Attribute::PAVObjectPath,
        VarID::AttributeTextureAndSeqObjectPath => Attribute::TextureAndSeqObjectPath,
        _ => {
            log::warn!(
                "Couldn't set attribute because {var_id:?} is not a valid attribute variable"
            );
            return;
        }
    };

    match id {
        Attribute::Timestamp | Attribute::UniverseBuild => {
            // It doesn't make sense to set these.
        }
        _ => {
            if let Err(why) = database.attrib_set(id, value) {
                log::warn!("Couldn't set attribute in database. {why:?}");
            }
        }
    }
}
