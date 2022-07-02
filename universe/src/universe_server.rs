use aw_core::*;

use crate::{client, Client};
use std::{
    cell::RefCell,
    net::{Ipv4Addr, SocketAddr, TcpListener},
};

pub struct UniverseServer {
    ip_address: Ipv4Addr,
    port: u16,
    clients: Vec<Client>,
    listener: TcpListener,
}

impl UniverseServer {
    pub fn new(addr: Ipv4Addr, port: u16) -> Self {
        let mut listener =
            TcpListener::bind(SocketAddr::new(std::net::IpAddr::V4(addr), port)).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            ip_address: addr,
            port,
            clients: Vec::new(),
            listener,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.accept_new_clients();
            self.service_clients();
            self.purge_dead_clients();
        }
    }

    fn accept_new_clients(&mut self) {
        while let Ok((stream, addr)) = self.listener.accept() {
            let client = Client::new(AWConnection::new(AWProtocol::new(stream)));
            self.clients.push(client);
        }
    }

    fn service_clients(&mut self) {
        let client_count = self.clients.len();
        for i in 0..client_count {
            let messages = self.clients[i].connection.recv();
            self.handle_messages(messages, i);
        }
        // We should not be adding or removing clients during this time.
        assert!(client_count == self.clients.len());
    }

    fn purge_dead_clients(&mut self) {
        self.clients = self.clients.drain(..).filter(|x| !x.dead).collect();
    }

    fn handle_messages(&mut self, messages: Vec<ProtocolMessage>, client_num: usize) {
        for message in messages {
            println!("{message:?}");
            match message {
                ProtocolMessage::Packet(packet) => {
                    self.handle_packet(&packet, client_num);
                }
                ProtocolMessage::Disconnect => {
                    self.clients[client_num].dead = true;
                }
                ProtocolMessage::StreamKey(_) => todo!(),
            }
        }
    }

    fn handle_packet(&mut self, packet: &AWPacket, client_num: usize) {
        match packet.get_opcode() {
            PacketType::PublicKeyRequest => self.handle_public_key_request(client_num, &packet),
            PacketType::StreamKeyResponse => self.handle_stream_key_response(client_num, &packet),
            _ => {
                println!("Unhandled packet {packet:?}");
            }
        }
    }

    fn send_attribs(&mut self, client_num: usize) {
        let client = &mut self.clients[client_num];
        let mut packet = AWPacket::new(PacketType::Attributes);
        packet.set_header_0(0);
        packet.set_header_1(0);

        // TODO: replace with real data
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_AllowTourists,
            "y".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_BetaBrowser,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_UniverseBuild,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_CitizenChanges,
            "y".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_SearchTabURL,
            "".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_Timestamp,
            "1234".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_WelcomeMessage,
            "WELCOME".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_BetaWorld,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_MinimumBrowser,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_LatestWorld,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_DefaultStartWorld,
            "".to_string(),
        ));
        packet.add_var(AWPacketVar::String(VarID::Attrib_Userlist, "y".to_string()));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_NotepadTabURL,
            "".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_MinimumBrowser,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_LatestBrowser,
            "0".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_UnknownBilling7,
            "".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_BillingMethod,
            "".to_string(),
        ));
        packet.add_var(AWPacketVar::String(
            VarID::Attrib_BillingUnknown9,
            "".to_string(),
        ));

        client.connection.send(packet);
    }

    fn handle_public_key_request(&mut self, client_num: usize, packet: &AWPacket) {
        let client = &mut self.clients[client_num];
        let key = client
            .rsa
            .encode_public_key()
            .expect("Public key was missing.");

        let mut packet = AWPacket::new(PacketType::PublicKeyResponse);
        packet.add_var(AWPacketVar::Data(VarID::EncryptionKey, key));
        client.connection.send(packet);
    }

    fn handle_stream_key_response(&mut self, client_num: usize, packet: &AWPacket) {
        let client = &mut self.clients[client_num];
        if let Some(encrypted_a4_key) = packet.get_data(VarID::EncryptionKey) {
            if let Ok(a4_key) = client.rsa.decrypt_private(&encrypted_a4_key) {
                client.connection.set_recv_key(&a4_key);
                self.send_attribs(client_num);
            }
        }
    }
}
