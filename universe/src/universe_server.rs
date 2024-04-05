use aw_core::*;

use crate::{
    client::ClientInfo,
    configuration,
    database::Database,
    get_conn, packet_handler,
    tabs::{regenerate_contact_list, regenerate_player_list, regenerate_world_list},
    universe_connection::{UniverseConnectionID, UniverseConnections},
    universe_license::LicenseGenerator,
    UniverseConnection,
};
use std::sync::Arc;
use std::{
    collections::HashMap,
    net::{SocketAddrV4, TcpListener},
};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread::sleep,
    time::Duration,
};

pub struct UniverseServer {
    pub config: configuration::UniverseConfig,
    pub license_generator: LicenseGenerator,
    pub connections: UniverseConnections,
    pub database: Database,
    listener: TcpListener,
}

impl UniverseServer {
    pub fn new(config: configuration::Config) -> Result<Self, String> {
        let database = Database::new(config.mysql, &config.universe)?;

        // The Universe server provides a license to incoming clients, which must contain information
        // about the IP address that the client is connecting to. This could be different from the
        // IP address that the server is actually bound to (e.g. bound to 0.0.0.0 while clients connect
        // to some other IP).
        let bind_socket = SocketAddrV4::new(config.universe.bind_ip, config.universe.port);
        let license_socket_addr =
            SocketAddrV4::new(config.universe.license_ip, config.universe.port);

        let listener = TcpListener::bind(bind_socket).unwrap();
        listener.set_nonblocking(true).unwrap();

        Ok(Self {
            config: config.universe,
            license_generator: LicenseGenerator::new(&license_socket_addr),
            connections: UniverseConnections::new(),
            database,
            listener,
        })
    }

    pub fn run(&mut self) {
        log::info!(
            "Starting universe on {}:{}. Providing licenses for {}. Protocol version {}.",
            self.config.bind_ip,
            self.config.port,
            self.config.license_ip,
            Self::protocol_version(),
        );

        let running = Arc::new(AtomicBool::new(true));

        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");

        while running.load(Ordering::SeqCst) {
            self.accept_new_clients();
            self.service_clients();
            self.remove_dead_clients();
            self.connections.send_tab_updates();
            self.connections.send_heartbeats();
            sleep(Duration::from_millis(1));
        }

        log::info!("Shutting down universe.");
    }

    fn protocol_version() -> &'static str {
        #[cfg(feature = "protocol_v4")]
        return "4";

        #[cfg(feature = "protocol_v6")]
        return "6";
    }

    fn accept_new_clients(&mut self) {
        while let Ok((stream, addr)) = self.listener.accept() {
            let conn = UniverseConnection::new(AWConnection::new(AWProtocol::new(stream)));
            self.connections.add_connection(conn);
            log::info!("{} connected.", addr.ip());
        }
    }

    fn service_clients(&mut self) {
        // Collect all new messages from clients
        let messages: HashMap<UniverseConnectionID, Vec<ProtocolMessage>> = self
            .connections
            .iter()
            .filter_map(|(&id, conn)| {
                let messages = conn.recv();
                if messages.is_empty() {
                    return None;
                }

                Some((id, messages))
            })
            .collect();

        // Handle all messages
        for (cid, messages) in messages {
            self.handle_messages(messages, cid);
        }
    }

    pub fn remove_dead_clients(&mut self) {
        let disconnected_conn_ids = self.connections.disconnected_cids();
        if disconnected_conn_ids.is_empty() {
            return;
        }
        for cid in &disconnected_conn_ids {
            let conn = get_conn!(self, *cid, "remove_dead_clients");
            log::info!("Removed client {}", conn.addr().ip());
        }

        // Figure out whether the player lists need to be remade, and remake them if so.
        let mut regen_player_lists = false;
        for cid in &disconnected_conn_ids {
            let Some(conn) = self.connections.get_connection(*cid) else {
                continue;
            };
            if conn.is_player() {
                regen_player_lists = true;
                break;
            };
        }

        // Figure out whether the world lists need to be remade, and remake them if so.
        let mut regen_world_lists = false;
        for cid in &disconnected_conn_ids {
            let Some(conn) = self.connections.get_connection(*cid) else {
                continue;
            };
            if let Some(ClientInfo::WorldServer(_)) = &conn.client {
                regen_world_lists = true;
                break;
            }
        }

        // Discard all the clients that have disconnected.
        self.connections.remove_disconnected();

        if regen_player_lists {
            for cid in self.connections.cids() {
                regenerate_player_list(self, cid);
                // This could be done only on the removed player's contacts
                regenerate_contact_list(self, cid);
            }
        }

        if regen_world_lists {
            for cid in self.connections.cids() {
                regenerate_world_list(self, cid);
            }
        }
    }

    fn handle_messages(&mut self, messages: Vec<ProtocolMessage>, cid: UniverseConnectionID) {
        for message in messages {
            match message {
                ProtocolMessage::Packet(packet) => {
                    self.handle_packet(&packet, cid);
                }
                ProtocolMessage::Disconnect => {
                    if let Some(conn) = self.connections.get_connection_mut(cid) {
                        conn.disconnect()
                    }
                }
                ProtocolMessage::StreamKey(_)
                | ProtocolMessage::Encrypt(_)
                | ProtocolMessage::PacketGroup(_) => {
                    panic!("Should not receive these message types on this end.");
                }
            }
        }
    }

    fn handle_packet(&mut self, packet: &AWPacket, cid: UniverseConnectionID) {
        log::trace!("Handling packet {packet:?}");
        match packet.get_opcode() {
            PacketType::PublicKeyRequest => packet_handler::public_key_request(self, cid),
            PacketType::StreamKeyResponse => packet_handler::stream_key_response(self, cid, packet),
            PacketType::PublicKeyResponse => packet_handler::public_key_response(self, cid, packet),
            PacketType::Login => packet_handler::login(self, cid, packet),
            PacketType::Heartbeat => packet_handler::heartbeat(self, cid),
            PacketType::WorldServerStart => packet_handler::world_server_start(self, cid, packet),
            PacketType::UserList => packet_handler::user_list(self, cid, packet),
            PacketType::AttributeChange => packet_handler::attribute_change(self, cid, packet),
            PacketType::CitizenNext => packet_handler::citizen_next(self, cid, packet),
            PacketType::CitizenPrev => packet_handler::citizen_prev(self, cid, packet),
            PacketType::CitizenLookupByName => {
                packet_handler::citizen_lookup_by_name(self, cid, packet)
            }
            PacketType::CitizenLookupByNumber => {
                packet_handler::citizen_lookup_by_number(self, cid, packet)
            }
            PacketType::CitizenChange => packet_handler::citizen_change(self, cid, packet),
            PacketType::LicenseAdd => packet_handler::license_add(self, cid, packet),
            PacketType::LicenseByName => packet_handler::license_by_name(self, cid, packet),
            PacketType::LicenseNext => packet_handler::license_next(self, cid, packet),
            PacketType::LicensePrev => packet_handler::license_prev(self, cid, packet),
            PacketType::LicenseChange => packet_handler::license_change(self, cid, packet),
            PacketType::WorldStart => packet_handler::world_start(self, cid, packet),
            PacketType::WorldStop => packet_handler::world_stop(self, cid, packet),
            PacketType::WorldList => packet_handler::world_list(self, cid, packet),
            PacketType::WorldLookup => packet_handler::world_lookup(self, cid, packet),
            PacketType::Identify => packet_handler::identify(self, cid, packet),
            PacketType::WorldStatsUpdate => packet_handler::world_stats_update(self, cid, packet),
            PacketType::CitizenAdd => packet_handler::citizen_add(self, cid, packet),
            PacketType::ContactAdd => packet_handler::contact_add(self, cid, packet),
            PacketType::TelegramSend => packet_handler::telegram_send(self, cid, packet),
            PacketType::TelegramGet => packet_handler::telegram_get(self, cid, packet),
            PacketType::SetAFK => packet_handler::set_afk(self, cid, packet),
            PacketType::ContactConfirm => packet_handler::contact_confirm(self, cid, packet),
            PacketType::ContactList => packet_handler::contact_list(self, cid, packet),
            PacketType::Join => packet_handler::join(self, cid, packet),
            PacketType::JoinReply => packet_handler::join_reply(self, cid, packet),
            PacketType::Botgram => packet_handler::botgram(self, cid, packet),
            PacketType::Immigrate => packet_handler::immigrate(self, cid, packet),
            PacketType::ContactDelete => packet_handler::contact_delete(self, cid, packet),
            PacketType::ContactChange => packet_handler::contact_change(self, cid, packet),
            _ => {
                log::warn!("Unhandled packet {packet:?}");
            }
        }
    }
}
