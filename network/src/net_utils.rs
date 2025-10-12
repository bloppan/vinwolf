use quinn::{ClientConfig, Endpoint, TransportConfig, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};
use rustls::{Error as RustlsError, SignatureScheme, DistinguishedName};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn base64_decode(input: &str) -> Result<Vec<u8>> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    let mut padding = 0u8;

    for &b in input.as_bytes() {
        if b == b'=' {
            padding += 1;
            continue;
        }
        if let Some(idx) = table.iter().position(|&c| c == b) {
            buffer = (buffer << 6) | idx as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                output.push(((buffer >> bits) & 0xff) as u8);
            }
        } else {
            return Err("Invalid base64 character".into());
        }
    }

    if padding > 2 {
        return Err("Invalid padding".into());
    }

    Ok(output)
}

pub fn parse_pem_certs(pem_data: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
    let pem_str = std::str::from_utf8(pem_data)?;
    let lines = pem_str.lines().map(|l| l.trim()).collect::<Vec<_>>();
    let mut certs = Vec::new();
    let mut collecting = false;
    let mut b64_lines = Vec::new();

    for line in lines {
        if line == "-----BEGIN CERTIFICATE-----" {
            collecting = true;
            b64_lines.clear();
        } else if line == "-----END CERTIFICATE-----" {
            if collecting {
                let b64 = b64_lines.join("");
                let der = base64_decode(&b64)?;
                certs.push(CertificateDer::from(der));
                collecting = false;
            }
        } else if collecting && !line.is_empty() {
            b64_lines.push(line);
        }
    }

    Ok(certs)
}

pub fn parse_pem_private_key(pem_data: &[u8]) -> Result<PrivateKeyDer<'static>> {
    let pem_str = std::str::from_utf8(pem_data)?;
    let lines = pem_str.lines().map(|l| l.trim()).collect::<Vec<_>>();
    let mut collecting = false;
    let mut b64_lines = Vec::new();

    for line in lines {
        if line == "-----BEGIN PRIVATE KEY-----" {
            collecting = true;
            b64_lines.clear();
        } else if line == "-----END PRIVATE KEY-----" {
            if collecting {
                let b64 = b64_lines.join("");
                let der = base64_decode(&b64)?;
                let pkcs8 = PrivatePkcs8KeyDer::from(der);
                return Ok(PrivateKeyDer::Pkcs8(pkcs8));
            }
        } else if collecting && !line.is_empty() {
            b64_lines.push(line);
        }
    }

    Err("No private key found".into())
}

