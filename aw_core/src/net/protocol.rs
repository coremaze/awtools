//! Networking protocol implementation
use crate::crypt_a4::AWCryptA4;
use crate::net::packet::{AWPacket, DeserializeError, PacketType};
use crate::ReasonCode;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// State of an instance of the AW protocol.
pub struct AWProtocol {
    stream: TcpStream,
    data: Vec<u8>,
    send_cipher: AWCryptA4,
    should_encrypt: bool,
    recv_cipher: Option<AWCryptA4>,
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
            send_cipher: AWCryptA4::new(),
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

    /// Set the key to receive data (i.e. the key the other end of the connection is using).
    pub fn set_recv_key(&mut self, key: &[u8]) {
        self.recv_cipher = Some(AWCryptA4::from_key(key));
    }

    /// Get the key for this side of the connection.
    pub fn get_send_key(&self) -> Vec<u8> {
        self.send_cipher.get_key()
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
            match packet.get_opcode() {
                PacketType::PublicKeyResponse
                | PacketType::StreamKeyResponse
                | PacketType::Attributes
                | PacketType::Login
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
            Err(_) => {
                // Failed to deserialize packet
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
        }

        drop(self);
    }

    fn handle_messages(&mut self) {
        if let Ok(message) = self.outbound_packets.try_recv() {
            match message {
                ProtocolMessage::Packet(packet) => {
                    if self.send(&mut [packet], true).is_err() {
                        self.inbound_packets.send(ProtocolMessage::Disconnect).ok();
                        self.dead = true;
                    }
                }
                ProtocolMessage::PacketGroup(mut packets) => {
                    if self.send(&mut packets, true).is_err() {
                        self.inbound_packets.send(ProtocolMessage::Disconnect).ok();
                        self.dead = true;
                    }
                }
                ProtocolMessage::StreamKey(key) => {
                    self.recv_cipher = Some(AWCryptA4::from_key(&key));
                    // There may be data that has already been sent, so we need to decrypt it now.
                    // self.recv_cipher.as_mut().unwrap().decrypt_in_place(&mut self.data);
                }
                ProtocolMessage::Encrypt(should) => {
                    self.encrypt_data(should);
                }
                ProtocolMessage::Disconnect => {
                    self.dead = true;
                }
            }
        }
    }

    fn handle_inbound_packets(&mut self) {
        if self.needs_action() {
            match self.recv_next_packet() {
                Some(packet) => {
                    self.last_packet_type = Some(packet.get_opcode());
                    if self
                        .inbound_packets
                        .send(ProtocolMessage::Packet(packet))
                        .is_err()
                    {
                        self.dead = true;
                    }
                }
                None => {
                    self.inbound_packets.send(ProtocolMessage::Disconnect).ok();
                    self.dead = true;
                }
            }
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
    use std::sync::mpsc::{channel, Receiver, Sender};
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
