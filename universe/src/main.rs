use aw_core::*;
use std::{net::Ipv4Addr, str::FromStr};

mod client;
pub use client::{Client, ClientType};
mod universe_server;
pub use universe_server::UniverseServer;
mod attributes;
pub mod license;
pub use attributes::{send_attributes, Attribute};
pub mod config;
pub mod packet_handler;

fn main() {
    match config::UniverseConfig::get() {
        Ok(config) => {
            UniverseServer::new(config).run();
        }
        Err(err) => {
            eprintln!("Could not get universe configuration: {}", err.to_string());
        }
    }
}
