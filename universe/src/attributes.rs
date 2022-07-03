use crate::{AWPacket, AWPacketVar, Client, PacketType, VarID};
pub enum Attribute {
    AllowTourists = 0,
    UnknownBilling1 = 1,
    BetaBrowser = 2,
    MinimumBrowser = 3,
    LatestBrowser = 4,
    UniverseBuild = 5,
    CitizenChanges = 6,
    UnknownBilling7 = 7,
    BillingMethod = 8,
    BillingUnknown9 = 9,
    SearchTabURL = 10,
    Timestamp = 11,
    WelcomeMessage = 12,
    BetaWorld = 13,
    MinimumWorld = 14,
    LatestWorld = 15,
    DefaultStartWorld = 16,
    Userlist = 17,
    NotepadTabURL = 18,
    MailTemplate = 19,
    MailFile = 20,
    MailCommand = 21,
    PAVObjectPath = 22,
    UnknownUniverseSetting = 23,
}

pub fn send_attributes(client: &Client) {
    let mut packet = AWPacket::new(PacketType::Attributes);
    packet.set_header_0(0);
    packet.set_header_1(0);

    // TODO: replace with real data
    packet.add_var(AWPacketVar::String(
        VarID::AttributeAllowTourists,
        "y".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBetaBrowser,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUniverseBuild,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeCitizenChanges,
        "y".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeSearchTabURL,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeTimestamp,
        "1234".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeWelcomeMessage,
        "WELCOME".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBetaWorld,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeMinimumBrowser,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeLatestWorld,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeDefaultStartWorld,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUserlist,
        "y".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeNotepadTabURL,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeMinimumBrowser,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeLatestBrowser,
        "0".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeUnknownBilling7,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBillingMethod,
        "".to_string(),
    ));
    packet.add_var(AWPacketVar::String(
        VarID::AttributeBillingUnknown9,
        "".to_string(),
    ));

    client.connection.send(packet);
}
