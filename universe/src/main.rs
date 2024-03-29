use aw_core::*;

mod client;
pub use client::UniverseConnection;
mod universe_server;
pub use universe_server::UniverseServer;
pub mod attributes;
pub mod universe_license;
pub use attributes::send_attributes;
mod database;
pub mod packet_handler;
pub mod tabs;
pub mod telegram;
pub mod world;

mod configuration;

use env_logger::Builder;
pub use log::{debug, error, info, trace, warn};

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, value_parser, default_value_t = log::LevelFilter::Info)]
    /// Verbosity of logging: <off | error | warn | info | debug | trace>
    log_level: log::LevelFilter,
}

fn init_logging(level: log::LevelFilter) {
    let mut builder = Builder::new();
    builder.filter_level(level);
    builder.init();
}

fn main() {
    let args = Args::parse();
    init_logging(args.log_level);

    match configuration::Config::get_interactive() {
        Ok(config) => {
            start_universe(config);
        }
        Err(err) => {
            log::error!("Could not get universe configuration: {err}");
        }
    }
}

fn start_universe(config: configuration::Config) {
    match UniverseServer::new(config) {
        Ok(mut universe) => {
            universe.run();
        }
        Err(err) => {
            log::error!("Could not create universe: {err}");
        }
    }
}
