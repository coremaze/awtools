use aw_core::*;

use crate::{
    client::{Client, ClientManager},
    config,
    license::LicenseGenerator,
    packet_handler,
};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

pub struct UniverseServer {
    config: config::UniverseConfig,
    license_generator: LicenseGenerator,
    client_manager: ClientManager,
    listener: TcpListener,
}

impl UniverseServer {
    pub fn new(config: config::UniverseConfig) -> Self {
        let ip = SocketAddrV4::new(config.ip, config.port);
        let listener = TcpListener::bind(&ip).unwrap();
        listener.set_nonblocking(true).unwrap();

        Self {
            config,
            license_generator: LicenseGenerator::new(&ip),
            client_manager: Default::default(),
            listener,
        }
    }

    pub fn run(&mut self) {
        println!(
            "Starting universe on {}:{}",
            self.config.ip, self.config.port
        );
        loop {
            self.accept_new_clients();
            self.service_clients();
            self.client_manager.remove_dead_clients();
        }
    }

    fn accept_new_clients(&mut self) {
        while let Ok((stream, _addr)) = self.listener.accept() {
            let client = Client::new(AWConnection::new(AWProtocol::new(stream)));
            self.client_manager.add_client(client);
        }
    }

    fn service_clients(&mut self) {
        for client in self.client_manager.clients() {
            let messages = client.connection.recv();
            self.handle_messages(messages, client);
        }
    }

    fn handle_messages(&self, messages: Vec<ProtocolMessage>, client: &Client) {
        for message in messages {
            match message {
                ProtocolMessage::Packet(packet) => {
                    self.handle_packet(&packet, client);
                }
                ProtocolMessage::Disconnect => {
                    client.kill();
                }
                ProtocolMessage::StreamKey(_) | ProtocolMessage::Encrypt(_) => {}
            }
        }
    }

    fn handle_packet(&self, packet: &AWPacket, client: &Client) {
        match packet.get_opcode() {
            PacketType::PublicKeyRequest => packet_handler::public_key_request(client),
            PacketType::StreamKeyResponse => packet_handler::stream_key_response(client, packet),
            PacketType::PublicKeyResponse => packet_handler::public_key_response(client, packet),
            PacketType::Login => packet_handler::login(
                client,
                packet,
                &self.client_manager,
                &self.license_generator,
            ),
            _ => {
                println!("Unhandled packet {packet:?}");
            }
        }
    }
}
