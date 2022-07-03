use aw_core::*;
use std::{net::Ipv4Addr, str::FromStr};

mod client;
pub use client::{Client, ClientType};
mod universe_server;
pub use universe_server::UniverseServer;
mod attributes;
pub mod license;
pub use attributes::{send_attributes, Attribute};
pub mod packet_handler;

fn main() {
    UniverseServer::new(Ipv4Addr::from_str("127.0.0.1").unwrap(), 6670).run();
}
