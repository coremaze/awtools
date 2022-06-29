use aw_core::{AWCryptRSA, AWRegLic, AWRegLicData, RSAKey};
use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser)]
struct Args {
    private_key_file: String,
    ip_address: Ipv4Addr,
    port: u16,
    output_file: String,
}

fn main() {
    let args = Args::parse();

    let key_bytes = std::fs::read(args.private_key_file).unwrap_or_else(|_| {
        println!("Could not read private key file.");
        std::process::exit(1);
    });

    let mut rsa = AWCryptRSA::new();
    rsa.decode_private_key(&key_bytes).unwrap_or_else(|_| {
        println!("Could not decode private key.");
        std::process::exit(1);
    });

    let mut reg_lic = AWRegLic::new(rsa);
    let reg_lic_data = AWRegLicData::default()
        .set_ip_address(&args.ip_address)
        .set_port(args.port.into())
        .set_name("aw")
        .set_expiration_time(i32::MAX);

    let encrypted_data = reg_lic
        .code_generate_base64(&reg_lic_data, RSAKey::Private)
        .unwrap_or_else(|_| {
            println!("Could not generate encrypted license.");
            std::process::exit(1);
        });

    std::fs::write(args.output_file, encrypted_data).unwrap_or_else(|_| {
        println!("Failed to write to output file.");
        std::process::exit(1);
    });
}
