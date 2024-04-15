use crate::{attributes, get_conn_mut, universe_connection::UniverseConnectionID, UniverseServer};
use aw_core::{AWPacket, VarID};

#[derive(Debug)]
enum StreamKeyResponseParamsError {
    EncryptedStreamCipherKey,
}

struct StreamKeyResponseParams {
    encrypted_stream_cipher_key: Vec<u8>,
}

impl TryFrom<&AWPacket> for StreamKeyResponseParams {
    type Error = StreamKeyResponseParamsError;

    fn try_from(value: &AWPacket) -> Result<Self, Self::Error> {
        let encrypted_stream_cipher_key = value
            .get_data(VarID::EncryptionKey)
            .ok_or(StreamKeyResponseParamsError::EncryptedStreamCipherKey)?;

        Ok(Self {
            encrypted_stream_cipher_key,
        })
    }
}

/// Handle a client sending the server its RC4 encryption key.
/// For all data afterwards, we use this key to decrypt traffic we receive.
pub fn stream_key_response(
    server: &mut UniverseServer,
    cid: UniverseConnectionID,
    packet: &AWPacket,
) {
    let conn = get_conn_mut!(server, cid, "stream_key_response");

    log::trace!("stream_key_response");

    let params = match StreamKeyResponseParams::try_from(packet) {
        Ok(params) => params,
        Err(why) => {
            log::debug!("Could not complete stream key response: {why:?}");
            return;
        }
    };

    // In AW 4 and 5, the stream cipher is RC4, in 6 it is AES
    let stream_key = match conn
        .rsa
        .decrypt_private(&params.encrypted_stream_cipher_key)
    {
        Ok(key) => key,
        Err(why) => {
            log::debug!("Could not decrypt client's stream key: {why:?}");
            return;
        }
    };

    conn.set_recv_key(&stream_key);
    log::trace!("stream_key_response send_attributes");
    attributes::send_attributes(conn, &server.database);
}
