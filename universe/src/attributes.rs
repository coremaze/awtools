use std::collections::HashMap;

use aw_db::DatabaseResult;

use crate::database::attrib::{AttribDB, Attribute};
use crate::database::UniverseDatabase;
use crate::timestamp::unix_epoch_timestamp_str;
use crate::{AWPacket, PacketType, UniverseConnection};

pub fn send_attributes(conn: &UniverseConnection, database: &UniverseDatabase) {
    let mut packet = AWPacket::new(PacketType::Attributes);
    packet.set_header_0(0);
    packet.set_header_1(0);

    log::trace!("get attributes");
    let attribs = get_attributes(database);
    log::trace!("get attributes done");

    packet.add_string(
        Attribute::AllowTourists,
        attribs
            .get(&Attribute::AllowTourists)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::BetaBrowser,
        attribs
            .get(&Attribute::BetaBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::UniverseBuild,
        attribs
            .get(&Attribute::UniverseBuild)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::CitizenChanges,
        attribs
            .get(&Attribute::CitizenChanges)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::SearchTabURL,
        attribs
            .get(&Attribute::SearchTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::Timestamp,
        attribs
            .get(&Attribute::Timestamp)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::WelcomeMessage,
        attribs
            .get(&Attribute::WelcomeMessage)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::BetaWorld,
        attribs
            .get(&Attribute::BetaWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::MinimumWorld,
        attribs
            .get(&Attribute::MinimumWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::LatestWorld,
        attribs
            .get(&Attribute::LatestWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::DefaultStartWorld,
        attribs
            .get(&Attribute::DefaultStartWorld)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::Userlist,
        attribs
            .get(&Attribute::Userlist)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::NotepadTabURL,
        attribs
            .get(&Attribute::NotepadTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::MinimumBrowser,
        attribs
            .get(&Attribute::MinimumBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::LatestBrowser,
        attribs
            .get(&Attribute::LatestBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::UnknownBilling7,
        attribs
            .get(&Attribute::UnknownBilling7)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(Attribute::BillingMethod, "".to_string());
    packet.add_string(
        Attribute::BillingUnknown9,
        attribs
            .get(&Attribute::BillingUnknown9)
            .unwrap_or(&String::new())
            .to_string(),
    );

    packet.add_string(
        Attribute::PAVObjectPath,
        attribs
            .get(&Attribute::PAVObjectPath)
            .unwrap_or(&String::new())
            .to_string(),
    );

    packet.add_data(
        Attribute::PAVObjectPasswordObfuscated,
        attribs
            .get(&Attribute::PAVObjectPasswordObfuscated)
            .unwrap_or(&String::new())
            .as_bytes()
            .to_vec(),
    );

    packet.add_string(
        Attribute::TextureAndSeqObjectPath,
        attribs
            .get(&Attribute::TextureAndSeqObjectPath)
            .unwrap_or(&String::new())
            .to_string(),
    );
    packet.add_string(
        Attribute::ObjectRefresh,
        attribs
            .get(&Attribute::ObjectRefresh)
            .unwrap_or(&String::new())
            .to_string(),
    );

    conn.send(packet);
}

pub fn get_attributes(database: &UniverseDatabase) -> HashMap<Attribute, String> {
    let mut result = match database.attrib_get() {
        DatabaseResult::Ok(attribs) => attribs,
        DatabaseResult::DatabaseError => {
            log::warn!("Unable to get universe attributes from database, but we are continuing since we can still provide time and universe build");
            HashMap::<Attribute, String>::new()
        }
    };

    // Not sure if the client actually cares about what build we use here
    result.insert(Attribute::UniverseBuild, "120".to_string());

    result.insert(Attribute::Timestamp, unix_epoch_timestamp_str());

    result
}

pub fn set_attribute(id: Attribute, value: &str, database: &UniverseDatabase) {
    match id {
        Attribute::Timestamp | Attribute::UniverseBuild => {
            // It doesn't make sense to set these.
        }
        _ => match database.attrib_set(id, value) {
            DatabaseResult::Ok(_) => {}
            DatabaseResult::DatabaseError => {
                log::warn!("Couldn't set attribute in database.")
            }
        },
    }
}
