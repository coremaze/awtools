use std::time::Duration;

use aw_core::ProtocolMessage;

use crate::{
    AwEvent, ConsoleMessageParams, HudCreateParams, LoginParams, LoginResult, ObjectInfo,
    QueryResult, SdkError, SdkResult, StateChangeParams, TeleportParams, WorldInfo,
    instance_conn::AwInstanceConnection,
    msg::{self, handler::from_world::attributes::WorldAttributes, out::hud::HudCreateResult},
    uni::handle_uni_packet,
    world::{World, handle_world_packet},
};

pub struct Session {
    pub session_id: u32,
    pub login_id: Option<u32>, // None for tourist
}

pub struct AwInstance {
    pub(crate) uni: AwInstanceConnection,
    pub(crate) session: Option<Session>,
    pub(crate) world: Option<World>,
    pub(crate) timeout: Duration,
}

impl AwInstance {
    pub fn new(domain: &str, port: u16) -> SdkResult<Self> {
        let uni = AwInstanceConnection::connect(domain, port)?;

        Ok(Self {
            uni,
            session: None,
            world: None,
            timeout: Duration::from_secs(5),
        })
    }

    pub fn tick(&mut self) -> Vec<AwEvent> {
        let uni_messages = self.uni.recv();
        let mut events = Vec::new();

        let backlog = self.uni.backlog_packets.drain(..).collect::<Vec<_>>();
        for packet in backlog {
            handle_uni_packet(self, packet, &mut events);
        }

        for message in uni_messages {
            match message {
                ProtocolMessage::Packet(packet) => handle_uni_packet(self, packet, &mut events),
                ProtocolMessage::Disconnect => {
                    // TODO: Handle disconnect properly - for now just log
                    eprintln!("Received disconnect message");
                    events.push(AwEvent::UniverseDisconnected);
                }
                ProtocolMessage::PacketGroup(_)
                | ProtocolMessage::StreamKey(_)
                | ProtocolMessage::Encrypt(_) => {
                    // TODO: Handle these message types properly - for now just log
                    eprintln!("Received unhandled message type");
                }
            }
        }

        if let Some(world) = &mut self.world {
            let world_messages = world.connection.recv();

            let backlog = world
                .connection
                .backlog_packets
                .drain(..)
                .collect::<Vec<_>>();
            for packet in backlog {
                handle_world_packet(self, packet, &mut events);
            }

            for message in world_messages {
                match message {
                    ProtocolMessage::Packet(packet) => {
                        handle_world_packet(self, packet, &mut events)
                    }
                    ProtocolMessage::Disconnect => {
                        // TODO: Handle disconnect properly - for now just log
                        eprintln!("Received world disconnect message");
                        self.world = None;
                        events.push(AwEvent::WorldDisconnected);
                    }
                    ProtocolMessage::PacketGroup(_)
                    | ProtocolMessage::StreamKey(_)
                    | ProtocolMessage::Encrypt(_) => {
                        // TODO: Handle these message types properly - for now just log
                        eprintln!("Received unhandled world message type");
                    }
                }
            }
        }
        events
    }

    pub fn login(&mut self, params: LoginParams) -> SdkResult<LoginResult> {
        msg::out::login::login(self, params)
    }

    pub fn enter(&mut self, world: &str, global: bool) -> SdkResult<()> {
        msg::out::enter::enter(self, world, global)
    }

    pub fn state_change(&mut self, params: StateChangeParams) -> SdkResult<()> {
        msg::out::state_change::state_change(self, params)
    }

    pub fn say(&mut self, message: &str) -> SdkResult<()> {
        msg::out::say::say(self, message)
    }

    pub fn world_lookup(&mut self, world: &str) -> SdkResult<WorldInfo> {
        msg::out::world_lookup::world_lookup(self, world)
    }

    pub fn world_attributes(&mut self) -> SdkResult<WorldAttributes> {
        let Some(world) = &mut self.world else {
            return Err(SdkError::NotConnectedToWorld);
        };

        let Some(attributes) = world.attributes.as_ref() else {
            return Err(SdkError::NotConnectedToWorld);
        };

        Ok(attributes.clone())
    }

    pub fn world_attribute_change(&mut self, attributes: &WorldAttributes) -> SdkResult<()> {
        msg::out::world_attribute::world_attribute_change(self, attributes)
    }

    pub fn hud_create(&mut self, params: HudCreateParams) -> SdkResult<HudCreateResult> {
        msg::out::hud::hud_create(self, params)
    }

    pub fn teleport(&mut self, params: TeleportParams) -> SdkResult<()> {
        msg::out::teleport::teleport(self, params)
    }

    pub fn console_message(&mut self, params: ConsoleMessageParams) -> SdkResult<()> {
        msg::out::console_message::console_message(self, params)
    }

    pub fn query(&mut self, sector_x: i32, sector_z: i32) -> SdkResult<QueryResult> {
        msg::out::query::query(self, sector_x, sector_z)
    }

    pub fn object_change(&mut self, object_info: ObjectInfo) -> SdkResult<()> {
        msg::out::object_change::object_change(self, object_info)
    }
}
