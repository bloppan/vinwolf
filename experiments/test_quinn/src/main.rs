mod client;
mod server;

use std::error::Error;
use client::run_client;
use server::run_server;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {

    run_client().await?;
    //run_server().await?;
    Ok(())
}

use base32::{Alphabet, encode};

const PUBKEY: [u8; 32] = [
    0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d,
    0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73,
    0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6,
    0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29,
];

//const PUBKEY: [u8; 32] = [57u8, 100, 133, 71, 244, 149, 234, 172, 144, 159, 120, 64, 51, 14, 115, 205, 248, 55, 248, 219, 138, 79, 88, 55, 40, 116, 44, 149, 138, 218, 200, 43];

/*const PUBKEY: [u8; 32] = [
    0xf8, 0xfd, 0x75, 0xc5, 0xd9, 0x52, 0xbf, 0x61,
    0x37, 0x7d, 0xa7, 0x4b, 0x8a, 0x03, 0x59, 0x51,
    0x89, 0x9c, 0x75, 0xe4, 0xc1, 0x52, 0xb9, 0x3f,
    0x34, 0x28, 0xec, 0x1b, 0x17, 0xa6, 0x48, 0x08,
];*/

fn dns_alt_name_from_pubkey(pk: &[u8; 32]) -> String {
    let b32 = encode(Alphabet::Rfc4648 { padding: false }, pk).to_lowercase();
    format!("e{b32}")
}

#[test]
fn alternative_name_test() {
    let result = dns_alt_name_from_pubkey(&PUBKEY);
    println!("result ascii: {}", result);
    println!("result hex: {}", utils::hex::encode(result));
}

#[test]
fn generate_key_pem() {
    let prefix: [u8; 16] = [
        0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
        0x04, 0x20,
    ];

    // Implement hex decode
    fn hex_decode(s: &str) -> Vec<u8> {
        let mut result = Vec::with_capacity(s.len() / 2);
        let mut chars = s.chars();
        while let (Some(high), Some(low)) = (chars.next(), chars.next()) {
            let high_val = match high.to_digit(16) {
                Some(v) => v as u8,
                None => panic!("Invalid hex char"),
            };
            let low_val = match low.to_digit(16) {
                Some(v) => v as u8,
                None => panic!("Invalid hex char"),
            };
            result.push((high_val << 4) | low_val);
        }
        result
    }

    // Ed25519 secret seed (32 bytes) desde hex
    let seed_hex = "996542becdf1e78278dc795679c825faca2e9ed2bf101bf3c4a236d3ed79cf59";
    let seed = hex_decode(seed_hex);

    // Concatenar prefix + seed
    let mut full_key = Vec::with_capacity(52);
    full_key.extend_from_slice(&prefix);
    full_key.extend_from_slice(&seed);

    // Implement base64 encode
    fn base64_encode(input: &[u8]) -> String {
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut output = String::new();
        let mut buffer = 0u32;
        let mut bits = 0u32;

        for &b in input {
            buffer = (buffer << 8) | b as u32;
            bits += 8;
            while bits >= 6 {
                bits -= 6;
                let idx = ((buffer >> bits) & 0x3f) as usize;
                output.push(table[idx] as char);
            }
        }

        if bits > 0 {
            let idx = ((buffer << (6 - bits)) & 0x3f) as usize;
            output.push(table[idx] as char);
        }

        let padding = (3 - input.len() % 3) % 3;
        for _ in 0..padding {
            output.push('=');
        }

        output
    }

    // Base64 encode
    let b64 = base64_encode(&full_key);

    // Generar PEM
    let pem = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----",
        b64
    );

    println!("{}", pem);
}