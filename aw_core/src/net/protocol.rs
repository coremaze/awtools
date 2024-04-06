//! Networking protocol implementation
use crate::net::packet::{AWPacket, DeserializeError, PacketType};
use crate::{AWCryptStream, StreamKeyError};
use crate::{PacketTypeResult, ReasonCode};
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[cfg(feature = "stream_cipher_aes")]
type StreamCipherType = crate::AWCryptAES;
#[cfg(feature = "stream_cipher_rc4")]
type StreamCipherType = crate::AWCryptA4;

/// State of an instance of the AW protocol.
pub struct AWProtocol {
    stream: TcpStream,
    data: Vec<u8>,
    send_cipher: StreamCipherType,
    should_encrypt: bool,
    recv_cipher: Option<StreamCipherType>,
    dead: bool,
    inbound_packets: Sender<ProtocolMessage>,
    outbound_packets: Receiver<ProtocolMessage>,
    other_inbound_packets: Option<Receiver<ProtocolMessage>>,
    other_outbound_packets: Option<Sender<ProtocolMessage>>,
    last_packet_type: Option<PacketType>,
}

impl AWProtocol {
    /// Create a new AWProtocol instance given a TCP stream that has already been established.
    pub fn new(stream: TcpStream) -> Self {
        let (outbound_packets_tx, outbound_packets_rx) = channel::<ProtocolMessage>();
        let (inbound_packets_tx, inbound_packets_rx) = channel::<ProtocolMessage>();

        Self {
            stream,
            data: Vec::new(),
            send_cipher: StreamCipherType::new(),
            should_encrypt: false,
            recv_cipher: None,
            dead: false,
            last_packet_type: None,
            inbound_packets: inbound_packets_tx,
            outbound_packets: outbound_packets_rx,
            other_inbound_packets: Some(inbound_packets_rx),
            other_outbound_packets: Some(outbound_packets_tx),
        }
    }

    /// Get the address of the connected peer
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.stream.peer_addr()
    }

    /// Set the key to receive data (i.e. the key the other end of the connection is using).
    pub fn set_recv_key(&mut self, key: &[u8]) -> Result<(), StreamKeyError> {
        self.recv_cipher = Some(StreamCipherType::from_key(key)?);
        Ok(())
    }

    /// Get the key for this side of the connection.
    pub fn get_send_key(&self) -> Vec<u8> {
        self.send_cipher.get_initial_random_buffer()
    }

    /// Specify whether transmitted data should be encrypted.
    pub fn encrypt_data(&mut self, should: bool) {
        self.should_encrypt = should;
    }

    /// Remove n oldest bytes from the recv buffer.
    pub fn remove_from_buf(&mut self, mut n: usize) {
        n = n.min(self.data.len());

        let new_buf = &self.data[n..];

        self.data = new_buf.to_vec();
    }

    /// Add bytes to the front of the recv buffer.
    pub fn insert_into_buf(&mut self, data: &[u8]) {
        let mut data2 = data.to_vec();
        std::mem::swap(&mut self.data, &mut data2);
        self.data.extend(data2);
    }

    /// Send packets.
    pub fn send(&mut self, packets: &mut [AWPacket], compression: bool) -> Result<(), ReasonCode> {
        for packet in packets.iter_mut() {
            let PacketTypeResult::PacketType(packet_type) = packet.get_type() else {
                continue;
            };
            match packet_type {
                PacketType::PublicKeyResponse
                | PacketType::StreamKeyResponse
                | PacketType::Attributes
                // When you are the server, this header should be 1, but
                // when you are the client, this header should be 2, or else
                // a normal universe server will return RC_MUST_UPGRADE
                // | PacketType::Login
                | PacketType::Tunnel => {
                    packet.set_header_1(1);
                }
                _ => {}
            }
        }

        // Serialize one or more packets
        let mut serialized_bytes = Vec::<u8>::new();
        for packet in packets.iter() {
            serialized_bytes.extend(packet.serialize().map_err(|_| ReasonCode::SendFailed)?);
        }

        // Try to compress the serialized packet
        let mut bytes_to_send = if compression {
            AWPacket::compress_if_needed(&serialized_bytes).map_err(|_| ReasonCode::SendFailed)?
        } else {
            serialized_bytes
        };

        // If the other end of the connection has been given our encryption key, we need to encrypt.
        if self.should_encrypt {
            bytes_to_send = self.send_cipher.encrypt(&bytes_to_send);
        }

        // Send the serialized packet.
        self.stream
            .write_all(&bytes_to_send)
            .map_err(|_| ReasonCode::SendFailed)?;

        Ok(())
    }

    /// Receive incoming bytes, return success
    pub fn recv(&mut self) -> Result<usize, String> {
        let mut buf = [0u8; 0x8000];
        // let mut buf = [0u8; 0x1]; // Can use this to stress-test recv
        if let Ok(bytes_read) = self.stream.read(&mut buf) {
            // Decrypt incoming bytes if we have a key.
            if let Some(cipher) = &mut self.recv_cipher {
                cipher.decrypt_in_place(&mut buf[..bytes_read]);
            }
            self.data.extend(&buf[..bytes_read]);

            if bytes_read == 0 {
                Err("Connection closed.".to_string())
            } else {
                Ok(bytes_read)
            }
        } else {
            Err("Could not receive bytes.".to_string())
        }
    }

    fn decompress_packet(&mut self, serialized_len: usize) {
        // Decompress it and replace the front of the recv buf with the decompressed packet.
        let compressed_data = &self.data[..serialized_len];
        if let Ok(decompressed) = AWPacket::decompress(compressed_data) {
            self.remove_from_buf(serialized_len);
            self.insert_into_buf(&decompressed);
        }
    }

    fn deserialize_packet(&mut self, serialized_len: usize) -> Result<Option<AWPacket>, String> {
        match AWPacket::deserialize(&self.data[..serialized_len]) {
            Ok((packet, consumed_bytes)) => {
                // Successfully deserialized a packet, now remove the data from the recv buf.
                self.remove_from_buf(consumed_bytes);
                return Ok(Some(packet));
            }
            Err(why) => {
                // Failed to deserialize packet
                log::debug!("Failed to deserialize packet: {why:?}");
                self.recv()?;
            }
        }
        Ok(None)
    }

    fn check_and_deserialize_packet(&mut self) -> Result<Option<AWPacket>, String> {
        match AWPacket::deserialize_check(&self.data) {
            // Received a packet that appears well formed, attempt to deserialize
            Ok(serialized_len) => {
                return self.deserialize_packet(serialized_len);
            }
            Err(err) => match err {
                DeserializeError::Length | DeserializeError::InvalidHeader => {
                    self.recv()?;
                }
                // Received a packet that is still compressed.
                DeserializeError::Compressed(serialized_len) => {
                    self.decompress_packet(serialized_len);
                }
            },
        }
        Ok(None)
    }

    /// Get next packet (if any) from the data which has been received.
    pub fn recv_next_packet(&mut self) -> Option<AWPacket> {
        loop {
            match self.check_and_deserialize_packet() {
                // If we get a packet, return it
                Ok(Some(packet)) => return Some(packet),
                // If there is an error that prevented getting a packet, stop
                Err(_) => return None,
                // If there was no error but no packet, try again
                _ => continue,
            }
        }
    }

    /// Returns whether there is anything to handle on a connection, including whether there has been an error.
    pub fn needs_action(&mut self) -> bool {
        // If we already have bytes, they need to be handled
        if !self.data.is_empty() {
            return true;
        }

        // If there are bytes on the socket, they need to be handled
        self.stream.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 1];
        let peek = self.stream.peek(&mut buf);
        self.stream.set_nonblocking(false).unwrap();

        // If the peek operation would block, that means it does not have data
        match peek {
            Err(x) if x.kind() == std::io::ErrorKind::WouldBlock => false,
            Ok(_) => true,
            _ => false,
        }
    }

    fn process_loop(mut self) {
        while !self.dead {
            self.handle_inbound_packets();

            // If we were just sent a stream key, we need to wait until it is decrypted and sent here.
            if let Some(PacketType::StreamKeyResponse) = self.last_packet_type {
                while self.recv_cipher.is_none() {
                    self.handle_messages();
                }
            }

            self.handle_messages();
            thread::sleep(Duration::from_millis(1));
        }

        self.inbound_packets.send(ProtocolMessage::Disconnect).ok();
        log::trace!("Ended stream {:?}", self.stream);
        drop(self);
    }

    pub fn send_or_kill(&mut self, packets: &mut [AWPacket], compression: bool) {
        if self.send(packets, compression).is_err() {
            self.kill();
        }
    }

    fn handle_messages(&mut self) {
        let message = match self.outbound_packets.try_recv() {
            Ok(message) => message,
            Err(_) => return,
        };

        match message {
            ProtocolMessage::Packet(packet) => {
                self.send_or_kill(&mut [packet], true);
            }
            ProtocolMessage::PacketGroup(mut packets) => {
                self.send_or_kill(&mut packets, true);
            }
            ProtocolMessage::StreamKey(key) => {
                match StreamCipherType::from_key(&key) {
                    Ok(mut stream_cipher) => {
                        // There may be data that has already been sent, so we need to decrypt it now.
                        stream_cipher.decrypt_in_place(&mut self.data);
                        self.recv_cipher = Some(stream_cipher);
                    }
                    Err(_) => self.kill(),
                }
            }
            ProtocolMessage::Encrypt(should) => {
                self.encrypt_data(should);
            }
            ProtocolMessage::Disconnect => {
                self.kill();
            }
        }
    }

    fn kill(&mut self) {
        self.dead = true;
    }

    fn handle_inbound_packets(&mut self) {
        if !self.needs_action() {
            // No work to do
            return;
        }

        // Get the next inbound packet
        let packet = match self.recv_next_packet() {
            Some(packet) => packet,
            None => {
                // Kill connection if could not get packet
                self.kill();
                return;
            }
        };

        if let PacketTypeResult::PacketType(packet_type) = packet.get_type() {
            self.last_packet_type = Some(packet_type);
        }

        let send_result = self.inbound_packets.send(ProtocolMessage::Packet(packet));

        if send_result.is_err() {
            self.kill();
        }
    }

    pub fn start_process_loop(mut self) -> (Sender<ProtocolMessage>, Receiver<ProtocolMessage>) {
        let outbound = self
            .other_outbound_packets
            .take()
            .expect("outbound packet channel already taken");
        let inbound = self
            .other_inbound_packets
            .take()
            .expect("inbound packet channel already taken");

        thread::spawn(|| {
            self.process_loop();
        });

        (outbound, inbound)
    }
}

#[derive(Debug)]
pub enum ProtocolMessage {
    Packet(AWPacket),
    PacketGroup(Vec<AWPacket>),
    Disconnect,
    StreamKey(Vec<u8>),
    Encrypt(bool),
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{AWPacketVar, VarID};
    use std::net::TcpListener;
    use std::sync::mpsc::channel;
    use std::thread;

    #[test]
    pub fn test1() {
        let listener = TcpListener::bind("0.0.0.0:1234").unwrap();

        let (tx, rx) = channel::<AWPacket>();

        // Start a new thread to send a packet to.
        let t1 = thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let mut proto = AWProtocol::new(stream);
                        let packet = proto.recv_next_packet().unwrap();
                        tx.send(packet).unwrap();
                        break;
                    }
                    Err(_) => assert!(false),
                }
            }
            drop(listener);
        });

        let stream = TcpStream::connect("127.0.0.1:1234").unwrap();
        let mut proto = AWProtocol::new(stream);

        // Construct a test packet.
        let mut packet = AWPacket::new(PacketType::AvatarAdd);
        packet.add_var(AWPacketVar::String(
            VarID::AFKStatus,
            "Hello, World!".to_string(),
        ));
        let data = (0..=255).collect::<Vec<u8>>();
        packet.add_var(AWPacketVar::Data(VarID::AttributeBetaWorld, data));

        // Send the test packet to other thread.
        let _ = proto.send(&mut [packet.clone()], true);

        // Get the packet that the other thread deserialized.
        let packet_2 = rx.recv().unwrap();

        t1.join().unwrap();

        // The deserialized packet should be the same as the packet originally sent.
        assert!(packet == packet_2);
    }
}
