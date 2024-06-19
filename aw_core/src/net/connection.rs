use crate::{AWPacket, AWPacketGroup, AWProtocol, ProtocolMessage};
use std::{
    net::SocketAddr,
    sync::mpsc::{Receiver, Sender},
};

#[derive(Debug)]
pub struct AWConnection {
    outbound: Sender<ProtocolMessage>,
    inbound: Receiver<ProtocolMessage>,
    a4_send_key: Vec<u8>,
    disconnected: bool,
    addr: SocketAddr,
}

impl AWConnection {
    pub fn new(protocol: AWProtocol, addr: SocketAddr) -> Self {
        let a4_send_key = protocol.get_send_key();

        let (outbound, inbound) = protocol.start_process_loop();

        Self {
            outbound,
            inbound,
            a4_send_key,
            disconnected: false,
            addr,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn send(&self, packet: AWPacket) {
        self.outbound.send(ProtocolMessage::Packet(packet)).ok();
    }

    pub fn send_group(&self, packets: AWPacketGroup) {
        self.outbound
            .send(ProtocolMessage::PacketGroup(packets.packets))
            .ok();
    }

    pub fn set_recv_key(&self, key: &[u8]) {
        self.outbound
            .send(ProtocolMessage::StreamKey(key.to_vec()))
            .ok();
    }

    pub fn get_send_key(&self) -> Vec<u8> {
        self.a4_send_key.clone()
    }

    pub fn encrypt_data(&self, should: bool) {
        self.outbound.send(ProtocolMessage::Encrypt(should)).ok();
    }

    pub fn recv(&self) -> Vec<ProtocolMessage> {
        let mut result = Vec::<ProtocolMessage>::new();
        while let Ok(message) = self.inbound.try_recv() {
            result.push(message);
        }
        result
    }

    pub fn disconnect(&mut self) {
        self.outbound.send(ProtocolMessage::Disconnect).ok();
        self.disconnected = true;
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }
}

impl Drop for AWConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}
