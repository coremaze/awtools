use crate::{AWPacket, AWProtocol, ProtocolMessage};
use std::sync::mpsc::{Receiver, Sender};

pub struct AWConnection {
    outbound: Sender<ProtocolMessage>,
    inbound: Receiver<ProtocolMessage>,
    a4_send_key: Vec<u8>,
}

impl AWConnection {
    pub fn new(protocol: AWProtocol) -> Self {
        let a4_send_key = protocol.get_send_key();
        let (outbound, inbound) = protocol.start_process_loop();

        Self { outbound, inbound, a4_send_key }
    }

    pub fn send(&mut self, packet: AWPacket) {
        self.outbound.send(ProtocolMessage::Packet(packet)).ok();
    }

    pub fn set_recv_key(&mut self, key: &[u8]) {
        self.outbound
            .send(ProtocolMessage::StreamKey(key.to_vec()))
            .ok();
    }

    pub fn get_send_key(&self) -> Vec<u8> {
        self.a4_send_key.clone()
    }

    pub fn encrypt_data(&mut self, should: bool) {
        self.outbound
            .send(ProtocolMessage::Encrypt(should))
            .ok();
    }

    pub fn recv(&mut self) -> Vec<ProtocolMessage> {
        let mut result = Vec::<ProtocolMessage>::new();
        loop {
            match self.inbound.try_recv() {
                Ok(message) => {
                    result.push(message);
                }
                Err(_) => {
                    break;
                }
            }
        }
        result
    }

    pub fn disconnect(&mut self) {
        self.outbound.send(ProtocolMessage::Disconnect).ok();
    }
}

impl Drop for AWConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}
