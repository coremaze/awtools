use std::net::Ipv4Addr;

use super::Config;

pub fn run_configurator() -> Config {
    let mut config = Config::default();

    // Universe

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

    // Mysql

    // Get hostname
    get_string(
        "Enter the hostname of the MySQL server.",
        &mut config.mysql.hostname,
    );

    // Get port
    get_port(
        "Enter the port of the MySQL server.",
        &mut config.mysql.port,
    );

    // Get username
    get_string(
        "Enter the username for the MySQL server.",
        &mut config.mysql.username,
    );

    // Get password
    get_string(
        "Enter the password for the MySQL server.",
        &mut config.mysql.password,
    );

    // Get database
    get_string(
        "Enter the database name to use on the MySQL server.",
        &mut config.mysql.database,
    );

    config
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
