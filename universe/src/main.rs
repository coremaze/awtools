use aw_core::*;
use std::{
    net::{Ipv4Addr, TcpListener},
    str::FromStr,
};

mod client;
pub use client::Client;
mod universe_server;
pub use universe_server::UniverseServer;

fn main() {
    UniverseServer::new(Ipv4Addr::from_str("0.0.0.0").unwrap(), 6670).run();
}
