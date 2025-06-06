use std::{
    net::TcpStream,
    time::{Duration, Instant},
};

use aw_core::{
    AWConnection, AWCryptRSA, AWPacket, AWPacketGroup, AWProtocol, PacketType, PacketTypeResult,
    ProtocolMessage, VarID,
};

use crate::{SdkError, SdkResult};

#[derive(Debug)]
pub struct AwInstanceConnection {
    pub domain: String,
    pub port: u16,
    conn: AWConnection,
    pub backlog_packets: Vec<AWPacket>,
}

impl AwInstanceConnection {
    pub fn connect(domain: &str, port: u16) -> SdkResult<Self> {
        let stream = TcpStream::connect(format!("{}:{}", domain, port))?;
        let addr = stream.peer_addr()?;
        let protocol = AWProtocol::new(stream)
            .map_err(|e| SdkError::protocol(format!("Failed to create protocol: {}", e)))?;
        let conn = AWConnection::new(protocol, addr);
        let my_rsa_key = AWCryptRSA::new();

        let mut conn = Self {
            domain: domain.to_string(),
            port,
            conn,
            backlog_packets: Vec::new(),
        };

        // Do public key request
        let public_key_request = AWPacket::new(PacketType::PublicKeyRequest);
        conn.send(public_key_request);

        let public_key_response = conn
            .wait_for_packet(PacketType::PublicKeyResponse, None)
            .ok_or_else(|| SdkError::protocol("Failed to receive public key response"))?;

        // Get the server's public RSA key
        let server_rsa_key = public_key_response
            .get_data(VarID::EncryptionKey)
            .ok_or_else(|| SdkError::missing_field("EncryptionKey"))?;
        let mut server_rsa = AWCryptRSA::new();
        server_rsa.decode_public_key(&server_rsa_key).map_err(|e| {
            SdkError::crypto(format!("Failed to decode server public key: {:?}", e))
        })?;
        server_rsa.randomize();

        // Encrypt our A4 key with the server's public key
        let my_a4_key = conn.get_send_key();
        let my_encrypted_a4_key = server_rsa
            .encrypt_public(&my_a4_key)
            .map_err(|e| SdkError::crypto(format!("Failed to encrypt A4 key: {:?}", e)))?;

        // Send the encrypted A4 key to the server
        let mut my_stream_pkt = AWPacket::new(PacketType::StreamKeyResponse);
        my_stream_pkt.add_data(VarID::EncryptionKey, my_encrypted_a4_key);
        conn.send(my_stream_pkt);
        conn.encrypt_data(true);

        // Send our public RSA key to the server
        let my_public_key = my_rsa_key
            .encode_public_key()
            .ok_or_else(|| SdkError::crypto("Failed to encode public key".to_string()))?;
        let mut my_public_key_pkt = AWPacket::new(PacketType::PublicKeyResponse);
        my_public_key_pkt.add_data(VarID::EncryptionKey, my_public_key);
        my_public_key_pkt.set_header_1(2);
        conn.send(my_public_key_pkt);

        let stream_key_response = conn
            .wait_for_packet(PacketType::StreamKeyResponse, None)
            .ok_or_else(|| SdkError::protocol("Failed to receive stream key response"))?;

        // Decrypt the server's stream key with our private RSA key
        let encrypted_server_stream_key = stream_key_response
            .get_data(VarID::EncryptionKey)
            .ok_or_else(|| SdkError::missing_field("EncryptionKey"))?;
        let server_stream_key = my_rsa_key
            .decrypt_private(&encrypted_server_stream_key)
            .map_err(|e| {
                SdkError::crypto(format!("Failed to decrypt server stream key: {:?}", e))
            })?;
        conn.set_recv_key(&server_stream_key);

        // eprintln!("Stream key set");

        Ok(conn)
    }

    pub fn send(&mut self, packet: AWPacket) {
        self.conn.send(packet);
    }

    pub fn recv(&mut self) -> Vec<ProtocolMessage> {
        self.conn.recv()
    }

    pub fn send_group(&mut self, packets: AWPacketGroup) {
        self.conn.send_group(packets);
    }

    pub fn set_recv_key(&mut self, key: &[u8]) {
        self.conn.set_recv_key(key);
    }

    pub fn get_send_key(&self) -> Vec<u8> {
        self.conn.get_send_key()
    }

    pub fn encrypt_data(&mut self, should: bool) {
        self.conn.encrypt_data(should);
    }

    pub fn disconnect(&mut self) {
        self.conn.disconnect();
    }

    pub fn is_disconnected(&self) -> bool {
        self.conn.is_disconnected()
    }

    pub fn wait_for_packet(
        &mut self,
        packet_type: PacketType,
        timeout: Option<Duration>,
    ) -> Option<AWPacket> {
        let start = Instant::now();
        while !self.is_disconnected() {
            // Check timeout
            if let Some(timeout) = timeout {
                if start.elapsed() > timeout {
                    return None;
                }
            }

            for message in self.recv() {
                match message {
                    ProtocolMessage::Packet(packet) => {
                        let pkt_type = packet.get_type();
                        if let PacketTypeResult::PacketType(pkt_type) = pkt_type {
                            if pkt_type == packet_type {
                                return Some(packet);
                            }
                        }
                        self.backlog_packets.push(packet);
                    }
                    ProtocolMessage::Disconnect => {
                        // TODO: Handle disconnect properly - for now just log
                        eprintln!("Received disconnect message in wait_for_packet");
                        return None;
                    }
                    ProtocolMessage::PacketGroup(_)
                    | ProtocolMessage::StreamKey(_)
                    | ProtocolMessage::Encrypt(_) => {
                        // TODO: Handle these message types properly - for now just log
                        eprintln!("Received unhandled message type in wait_for_packet");
                    }
                }
            }
        }
        None
    }
}
