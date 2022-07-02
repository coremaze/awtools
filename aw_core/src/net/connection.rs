use crate::{AWProtocol, ProtocolMessage, AWPacket};
use std::sync::mpsc::{Receiver, Sender};

pub struct AWConnection {
    outbound: Sender<ProtocolMessage>,
    inbound: Receiver<ProtocolMessage>,
}

impl AWConnection {
    pub fn new(protocol: AWProtocol) -> Self {
        let (outbound, inbound) = protocol.start_process_loop();

        Self {
            outbound,
            inbound,
        }
    }

    pub fn send(&mut self, packet: AWPacket) {
        self.outbound.send(ProtocolMessage::Packet(packet)).ok();
    }

    pub fn set_recv_key(&mut self, key: &[u8]) {
        println!("Sending key");
        self.outbound.send(ProtocolMessage::StreamKey(key.to_vec())).ok();
    }

    pub fn recv(&mut self) -> Vec<ProtocolMessage> {
        let mut result = Vec::<ProtocolMessage>::new();
        loop {
            match self.inbound.try_recv() {
                Ok(message) => {
                    result.push(message);
                },
                Err(_) => {
                    break;
                },
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