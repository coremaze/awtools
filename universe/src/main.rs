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
mod database;
pub mod packet_handler;

fn main() {
    match config::Config::get() {
        Ok(config) => {
            start_universe(config);
        }
        Err(err) => {
            eprintln!("Could not get universe configuration: {}", err.to_string());
        }
    }
}

fn start_universe(config: config::Config) {
    match UniverseServer::new(config) {
        Ok(mut universe) => {
            universe.run();
        }
        Err(err) => {
            eprintln!("Could not create universe: {err}");
        }
    }
}
