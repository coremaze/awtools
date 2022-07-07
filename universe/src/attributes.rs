use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::database::attrib::{AttribDB, Attribute};
use crate::database::Database;
use crate::{AWPacket, AWPacketVar, Client, PacketType, VarID};

pub fn send_attributes(client: &Client, database: &Database) {
    let mut packet = AWPacket::new(PacketType::Attributes);
    packet.set_header_0(0);
    packet.set_header_1(0);

    let attribs = get_attributes(database);

    packet.add_var(AWPacketVar::String(
        VarID::AttributeAllowTourists,
        attribs
            .get(&Attribute::AllowTourists)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBetaBrowser,
        attribs
            .get(&Attribute::BetaBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUniverseBuild,
        attribs
            .get(&Attribute::UniverseBuild)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeCitizenChanges,
        attribs
            .get(&Attribute::CitizenChanges)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeSearchTabURL,
        attribs
            .get(&Attribute::SearchTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeTimestamp,
        attribs
            .get(&Attribute::Timestamp)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeWelcomeMessage,
        attribs
            .get(&Attribute::WelcomeMessage)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBetaWorld,
        attribs
            .get(&Attribute::BetaWorld)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeMinimumWorld,
        attribs
            .get(&Attribute::MinimumWorld)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeLatestWorld,
        attribs
            .get(&Attribute::LatestWorld)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeDefaultStartWorld,
        attribs
            .get(&Attribute::DefaultStartWorld)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUserlist,
        attribs
            .get(&Attribute::Userlist)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeNotepadTabURL,
        attribs
            .get(&Attribute::NotepadTabURL)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeMinimumBrowser,
        attribs
            .get(&Attribute::MinimumBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeLatestBrowser,
        attribs
            .get(&Attribute::LatestBrowser)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUnknownBilling7,
        attribs
            .get(&Attribute::UnknownBilling7)
            .unwrap_or(&String::new())
            .to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBillingMethod,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBillingUnknown9,
        attribs
            .get(&Attribute::BillingUnknown9)
            .unwrap_or(&String::new())
            .to_string(),
    ));

    client.connection.send(packet);
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

pub fn set_attribute(var_id: VarID, value: &str, database: &Database) -> Result<(), ()> {
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
        VarID::AttributeUnknownUniverseSetting => Attribute::UnknownUniverseSetting,
        _ => {
            return Err(());
        }
    };

    database.attrib_set(id, value).map_err(|_| ())?;

    Ok(())
}
