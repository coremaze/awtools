use crate::{
    client::ClientInfo, database::ContactDB, get_conn, player::Player,
    universe_connection::UniverseConnectionID, UniverseServer,
};
use aw_core::{AWPacket, PacketType, ReasonCode, VarID};
use aw_db::DatabaseResult;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub fn botgram(server: &UniverseServer, cid: UniverseConnectionID, packet: &AWPacket) {
    let Some(params) = BotgramParams::from_packet(packet) else {
        return;
    };

    match params.botgram_type {
        BotgramType::Botgram => bot_botgram(server, cid, params),
        BotgramType::Invite => invite(server, cid, params),
    }
}

fn invite(server: &UniverseServer, cid: UniverseConnectionID, params: BotgramParams) {
    let source_conn = get_conn!(server, cid, "invite");

    let effective_source_id = source_conn
        .client
        .as_ref()
        .and_then(ClientInfo::citizen_id)
        .or_else(|| {
            source_conn
                .client
                .as_ref()
                .and_then(ClientInfo::player_info)
                .and_then(|x| x.privilege_id)
        });

    let Some(source_username) = source_conn
        .client
        .as_ref()
        .and_then(ClientInfo::player)
        .map(Player::username)
    else {
        send_botgram_response_rc(server, cid, ReasonCode::NotLoggedIn);
        return;
    };

    if !is_valid_uri(&params.message) {
        log::warn!(
            "{source_username} sent an invalid invite URI {:?}",
            &params.message
        );
        return;
    }

    let target_citizen_id = params.citizen_number;

    let DatabaseResult::Ok(invites_allowed) = server
        .database
        .contact_invites_allowed(target_citizen_id, effective_source_id.unwrap_or(0))
    else {
        log::error!("Could not complete invite due to database error");
        return;
    };

    if !invites_allowed {
        send_botgram_response_rc(server, cid, ReasonCode::JoinRefused);
        return;
    }

    // Invites go to anyone with the target ID, or to any bots using that as their privilege ID
    let mut target_cids = Vec::<UniverseConnectionID>::new();
    if let Some(target_cid) = server.connections.get_by_citizen_id(params.citizen_number) {
        target_cids.push(target_cid);
    };
    for (&cid, conn) in server.connections.iter() {
        let Some(priv_id) = conn.player_info().and_then(|p| p.privilege_id) else {
            continue;
        };
        if conn.is_bot() && priv_id == target_citizen_id {
            target_cids.push(cid);
        }
    }
    for target_cid in target_cids {
        let target_conn = get_conn!(server, target_cid, "invite");
        let mut forwarded_packet = AWPacket::new(PacketType::Botgram);
        forwarded_packet.add_uint(VarID::BotgramCitizenNumber, params.citizen_number);
        forwarded_packet.add_string(VarID::BotgramMessage, params.message.clone());
        forwarded_packet.add_uint(VarID::BotgramType, params.botgram_type as u32);
        forwarded_packet.add_uint(
            VarID::BotgramFromCitizenNumber,
            effective_source_id.unwrap_or(0),
        );
        forwarded_packet.add_string(VarID::BotgramFromUsername, source_username.clone());
        target_conn.send(forwarded_packet);
    }
    send_botgram_response_rc(server, cid, ReasonCode::Success);
}

fn bot_botgram(server: &UniverseServer, cid: UniverseConnectionID, params: BotgramParams) {
    let source_conn = get_conn!(server, cid, "invite");
    let effective_source_id = source_conn
        .client
        .as_ref()
        .and_then(ClientInfo::citizen_id)
        .or_else(|| {
            source_conn
                .client
                .as_ref()
                .and_then(ClientInfo::player_info)
                .and_then(|x| x.privilege_id)
        });

    let Some(source_username) = source_conn
        .client
        .as_ref()
        .and_then(ClientInfo::player)
        .map(Player::username)
    else {
        send_botgram_response_rc(server, cid, ReasonCode::NotLoggedIn);
        return;
    };

    let DatabaseResult::Ok(telegrams_allowed) = server
        .database
        .contact_telegrams_allowed(params.citizen_number, effective_source_id.unwrap_or(0))
    else {
        log::error!("Could not complete bot_botgram due to database error");
        return;
    };

    if !telegrams_allowed {
        send_botgram_response_rc(server, cid, ReasonCode::TelegramBlocked);
        return;
    }

    // Actual botgrams go to bots using the same privilege ID
    let mut target_cids = Vec::<UniverseConnectionID>::new();
    for (&cid, conn) in server.connections.iter() {
        let Some(priv_id) = conn.player_info().and_then(|p| p.privilege_id) else {
            continue;
        };
        if conn.is_bot() && priv_id == params.citizen_number {
            target_cids.push(cid);
        }
    }
    for target_cid in target_cids {
        let target_conn = get_conn!(server, target_cid, "bot_botgram");
        let mut forwarded_packet = AWPacket::new(PacketType::Botgram);
        forwarded_packet.add_uint(VarID::BotgramCitizenNumber, params.citizen_number);
        forwarded_packet.add_string(VarID::BotgramMessage, params.message.clone());
        forwarded_packet.add_uint(VarID::BotgramType, params.botgram_type as u32);
        forwarded_packet.add_uint(
            VarID::BotgramFromCitizenNumber,
            effective_source_id.unwrap_or(0),
        );
        forwarded_packet.add_string(VarID::BotgramFromUsername, source_username.clone());
        target_conn.send(forwarded_packet);
    }
    send_botgram_response_rc(server, cid, ReasonCode::Success);
}

fn send_botgram_response_rc(server: &UniverseServer, cid: UniverseConnectionID, rc: ReasonCode) {
    let conn = get_conn!(server, cid, "send_botgram_response_rc");
    let mut p = AWPacket::new(PacketType::BotgramResponse);
    p.add_uint(VarID::ReasonCode, rc.into());
    conn.send(p);
}

struct BotgramParams {
    citizen_number: u32,
    botgram_type: BotgramType,
    message: String,
}

impl BotgramParams {
    fn from_packet(packet: &AWPacket) -> Option<BotgramParams> {
        Some(BotgramParams {
            citizen_number: packet.get_uint(VarID::BotgramCitizenNumber)?,
            botgram_type: packet
                .get_uint(VarID::BotgramType)
                .and_then(BotgramType::from_u32)?,
            message: packet.get_string(VarID::BotgramMessage)?,
        })
    }
}

#[derive(FromPrimitive, Clone, Copy)]
enum BotgramType {
    Botgram = 0,
    Invite = 1,
}

fn is_valid_uri(uri: impl AsRef<str>) -> bool {
    let uri = uri.as_ref();

    // The URI should contain only ASCII
    if !uri.chars().all(|c| c.is_ascii()) {
        return false;
    }

    if uri.len() > 255 {
        return false;
    }

    // Protocol must be "aw://"
    let mut parts = uri.split("://");

    let Some(protocol) = parts.next() else {
        return false;
    };

    if protocol != "aw" {
        return false;
    }

    let Some(address_and_path) = parts.next() else {
        return false;
    };

    // URI should be of format "aw://PATH". If there are any more instances of "://", reject it.
    if parts.next().is_some() {
        return false;
    }

    let mut address_parts = address_and_path.split('/');

    // First element of the path should be the hostname and port.
    let Some(host_port) = address_parts.next() else {
        return false;
    };

    // The first element of the path should be formatted as HOSTNAME:PORT
    let mut host_port_parts = host_port.split(':');

    let Some(hostname) = host_port_parts.next() else {
        return false;
    };

    let Some(port_str) = host_port_parts.next() else {
        return false;
    };

    // There should be only one ":"
    if host_port_parts.next().is_some() {
        return false;
    }

    // Don't allow hostnames that are too short or too long
    if hostname.len() < 2 || hostname.len() > 127 {
        return false;
    }

    // Port should be a valid u16
    let Ok(port) = port_str.parse::<u16>() else {
        return false;
    };

    // Port should be exactly the same when re-encoded as a string, no leading zeroes
    if port.to_string() != port_str {
        return false;
    }

    let Some(world_name) = address_parts.next() else {
        return false;
    };

    // World name cannot be too short or too long
    if world_name.is_empty() || world_name.len() > 15 {
        return false;
    }

    // "raw" is required
    if address_parts.next() != Some("raw") {
        return false;
    }

    let Some(coords) = address_parts.next() else {
        return false;
    };

    // There should be no address parts after the coords
    if address_parts.next().is_some() {
        return false;
    }

    let mut coord_parts = coords.split(',');
    for _ in 0..4 {
        let Some(coord_str) = coord_parts.next() else {
            return false;
        };

        let Ok(coord) = coord_str.parse::<i32>() else {
            return false;
        };

        // Coord should be exactly the same when re-encoded as a string
        if coord.to_string() != coord_str {
            return false;
        }
    }

    // There should be only 4 coords
    if coord_parts.next().is_some() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_uri() {
        let valid_uri = "aw://example.com:1234/world/raw/-52,-7499,681,870";
        assert!(is_valid_uri(valid_uri));
    }

    #[test]
    fn test_invalid_protocol() {
        let invalid_uri = "http://example.com:1234/world/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_format() {
        let invalid_uri = "aw://example.com:1234/world/raw/-52,-7499,681,870/extra";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_hostname_length() {
        let invalid_uri = "aw://a:1234/world/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));

        let hostname = "a".repeat(128);

        let invalid_uri = format!("aw://{hostname}:1234/world/raw/-52,-7499,681,870");
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_port() {
        let invalid_uri = "aw://example.com:65536/world/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_world_name_length() {
        let invalid_uri = "aw://example.com:1234//raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));

        let invalid_uri = "aw://example.com:1234/aaaaaaaaaaaaaaaa/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_missing_raw() {
        let invalid_uri = "aw://example.com:1234/world/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_coord_format() {
        let invalid_uri = "aw://example.com:1234/world/raw/-52,-7499,681";
        assert!(!is_valid_uri(invalid_uri));

        let invalid_uri = "aw://example.com:1234/world/raw/-52,-7499,681,870,1234";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_invalid_coord_value() {
        let invalid_uri = "aw://example.com:1234/world/raw/-52,-7499,681,2147483648";
        assert!(!is_valid_uri(invalid_uri));
    }

    #[test]
    fn test_non_ascii_characters() {
        let invalid_uri = "aw://éxample.com:1234/world/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));

        let invalid_uri = "aw://example.com:1234/wörld/raw/-52,-7499,681,870";
        assert!(!is_valid_uri(invalid_uri));
    }
}
