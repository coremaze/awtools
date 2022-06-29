use aw_core::{AWCryptRSA, AWRegLic, RSAKey};
use clap::Parser;

#[derive(Parser)]
struct Args {
    input_file: String,
    public_key_file: String,
}

fn main() {
    let args = Args::parse();

    let license_text = std::fs::read_to_string(args.input_file).unwrap_or_else(|_| {
        println!("Could not read input file.");
        std::process::exit(1);
    });

    let key_bytes = std::fs::read(args.public_key_file).unwrap_or_else(|_| {
        println!("Could not read public key file.");
        std::process::exit(1);
    });

    let mut rsa = AWCryptRSA::new();
    rsa.decode_public_key(&key_bytes).unwrap_or_else(|_| {
        println!("Could not decode public key.");
        std::process::exit(1);
    });

    let mut reg_lic = AWRegLic::new(rsa);
    let license_data = reg_lic
        .code_process_base64(&license_text, RSAKey::Public)
        .unwrap_or_else(|err| {
            println!("{}", err);
            std::process::exit(1);
        });

    println!("{license_data}");
}
