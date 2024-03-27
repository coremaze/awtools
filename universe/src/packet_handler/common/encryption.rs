use crate::{attributes, client::UniverseConnectionID, get_conn_mut, UniverseServer};
use aw_core::{AWCryptRSA, AWPacket, PacketType, VarID};

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

/// Handle a client sending the server its RC4 encryption key.
/// For all data afterwards, we use this key to decrypt traffic we receive.
pub fn stream_key_response(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let conn = get_conn_mut!(server, cid, "stream_key_response");
    let database = &server.database;

    if let Some(encrypted_a4_key) = packet.get_data(VarID::EncryptionKey) {
        if let Ok(a4_key) = conn.rsa.decrypt_private(&encrypted_a4_key) {
            conn.set_recv_key(&a4_key);
            attributes::send_attributes(conn, database);
        }
    }
}

/// Handle a client sending the server its public RSA key.
/// We use it to share our RC4 key with the client.
/// All data the server sends afterwards should be encrypted with our RC4 key.
pub fn public_key_response(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let conn = get_conn_mut!(server, cid, "public_key_response");

    if let Some(rsa_key_bytes) = packet.get_data(VarID::EncryptionKey) {
        // Decode their public key
        let mut public_rsa = AWCryptRSA::default();
        public_rsa.randomize();
        if public_rsa.decode_public_key(&rsa_key_bytes).is_err() {
            return;
        }

        // Encrypt our RC4 key using the client's RSA key
        match public_rsa.encrypt_public(&conn.get_send_key()) {
            Ok(encrypted_a4) => {
                let mut response = AWPacket::new(PacketType::StreamKeyResponse);
                response.add_data(VarID::EncryptionKey, encrypted_a4);
                conn.send(response);
                conn.encrypt_data(true);
            }
            Err(e) => {
                println!("Failed to encrypt: {e:?}");
            }
        }
    }
}
