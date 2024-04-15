use crate::{get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::{AWPacket, PacketType, VarID};

/// Handle a client requesting the server's public RSA key.
/// We generate a new RSA key pair for each client since AW
/// versions prior to 7.0 use very weak RSA encryption.
/// We send the generated key pair to the client.
pub fn public_key_request(server: &mut UniverseServer, cid: UniverseConnectionID) {
    let conn = get_conn_mut!(server, cid, "public_key_request");

    let Some(key) = conn.rsa.encode_public_key() else {
        log::warn!("Failed to encode public key for client: {conn:?}");
        return;
    };

    let mut packet = AWPacket::new(PacketType::PublicKeyResponse);
    packet.add_data(VarID::EncryptionKey, key);
    conn.send(packet);
}
