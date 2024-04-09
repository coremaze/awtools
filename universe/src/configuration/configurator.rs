use std::net::Ipv4Addr;

use super::{config::SqliteConfig, Config, DatabaseType, MysqlConfig};

/// Interactive configurator to set up the Universe's toml file
pub fn run_configurator() -> Config {
    let mut config = Config::default();

    config_universe(&mut config);
    config_database(&mut config);

    config
}

/// Configure only the Universe-related settings
fn config_universe(config: &mut Config) {
    // Get bind_ip
    get_ip(
        "Enter the IP address that the universe server will be bound to.",
        &mut config.universe.bind_ip,
    );

    // Get license_ip
    get_ip(
        "Enter the IP address to use for licensing. This must be the IP address that clients connect to. If it is incorrect, clients will report error 471.",
        &mut config.universe.license_ip,
    );

    // Get port
    get_port(
        "Enter the port that the universe server will be bound to.",
        &mut config.universe.port,
    );
}

/// Configure the database settings, internal or external
fn config_database(config: &mut Config) {
    get_db_type(
        "Enter \"internal\" or \"external\" database.",
        &mut config.sql.database_type,
    );

    match config.sql.database_type {
        DatabaseType::External => config_mysql(&mut config.sql.mysql_config),
        DatabaseType::Internal => config_sqlite(&mut config.sql.sqlite_config),
    }
}

/// Configure the external database
fn config_mysql(config: &mut MysqlConfig) {
    // Get hostname
    get_string(
        "Enter the hostname of the MySQL server.",
        &mut config.hostname,
    );

    // Get port
    get_port("Enter the port of the MySQL server.", &mut config.port);

    // Get username
    get_string(
        "Enter the username for the MySQL server.",
        &mut config.username,
    );

    // Get password
    get_string(
        "Enter the password for the MySQL server.",
        &mut config.password,
    );

    // Get database
    get_string(
        "Enter the database name to use on the MySQL server.",
        &mut config.database,
    );
}

/// Configure the internal database
fn config_sqlite(config: &mut SqliteConfig) {
    // Get path
    get_string(
        "Enter path to the file to be created for the internal database.",
        &mut config.path,
    );
}

fn get_ip(message: &str, ip: &mut Ipv4Addr) {
    loop {
        println!("{} Default: {}", message, ip);
        let mut input = String::new();
        if let Err(why) = std::io::stdin().read_line(&mut input) {
            println!("Could not read input.");
            println!("{why}");
            continue;
        }

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            return;
        }

        if let Ok(parsed_ip) = input_trimmed.parse() {
            *ip = parsed_ip;
            return;
        } else {
            println!("Invalid IP address. Please try again.");
        }
    }
}

fn get_port(message: &str, default_port: &mut u16) {
    loop {
        println!("{} Default: {}", message, default_port);
        let mut port = String::new();
        if let Err(why) = std::io::stdin().read_line(&mut port) {
            println!("Could not read port number.");
            println!("{why}");
            continue;
        }

        let port_trimmed = port.trim();
        if port_trimmed.is_empty() {
            return;
        }

        if let Ok(parsed_port) = port_trimmed.parse::<u16>() {
            *default_port = parsed_port;
            return;
        } else {
            println!("Invalid port number. Please try again.");
        }
    }
}

fn get_string(message: &str, default_value: &mut String) {
    loop {
        println!("{} Default: {}", message, default_value);
        let mut input = String::new();
        if let Err(why) = std::io::stdin().read_line(&mut input) {
            println!("Could not read input.");
            println!("{why}");
            continue;
        }

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            return;
        }

        *default_value = input_trimmed.to_string();
        return;
    }
}

fn get_db_type(message: &str, default_value: &mut DatabaseType) {
    loop {
        let default_str = match default_value {
            DatabaseType::External => "External",
            DatabaseType::Internal => "Internal",
        };
        println!("{} Default: {}", message, default_str);
        let mut input = String::new();
        if let Err(why) = std::io::stdin().read_line(&mut input) {
            println!("Could not read input.");
            println!("{why}");
            continue;
        }

        let input_trimmed = input.trim();
        if input_trimmed.is_empty() {
            return;
        }

        match input.trim().to_lowercase().as_str() {
            "internal" => *default_value = DatabaseType::Internal,
            "external" => *default_value = DatabaseType::External,
            _ => {
                println!("Invalid database type. Choose internal or external.");
                continue;
            }
        }

        return;
    }
}
