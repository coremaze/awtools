use crate::{get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::{AWCryptRSA, AWPacket, PacketType, VarID};

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
