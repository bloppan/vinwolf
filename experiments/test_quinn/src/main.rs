use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::RootCertStore;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::Arc;
use anyhow::{Result, bail};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Explicitly set the ring provider
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| anyhow::anyhow!("Failed to install ring provider: "))?;

    // Node to connect to
    let node_alt_name = "elfaiiixcuzmzroa34lajwp52cdsucikaxdviaoeuvnygdi3imtba";
    let node_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 40005);

    // Genesis hash and ALPN protocol
    let genesis_hash = "b5af8eda";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    // Load certificate and key from PEM files
    let mut cert_file = std::io::Cursor::new(std::fs::read("cert.pem")?);
    let mut key_file = std::io::Cursor::new(std::fs::read("key.pem")?);

    // Parse certificates
    let certs: Vec<CertificateDer> = certs(&mut cert_file)
        .map(|result| result.map(CertificateDer::from))
        .collect::<Result<Vec<_>, _>>()?;
    if certs.is_empty() {
        bail!("No valid certificates found in cert.pem");
    }

    // Parse private key (assuming PKCS#8 format for Ed25519)
    let mut keys = pkcs8_private_keys(&mut key_file);
    let key = PrivateKeyDer::from(
        keys.next()
            .ok_or_else(|| anyhow::anyhow!("No valid private keys found in key.pem"))??,
    );

    // Configure rustls for QUIC
    let mut root_store = RootCertStore::empty();
    // For testing, trust the node's certificate (replace with proper validation in production)
    root_store.add(certs[0].clone())?;

    let mut client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_client_auth_cert(certs, key)?;

    client_crypto.alpn_protocols = vec![alpn_protocol];

    // Configure Quinn client for QUIC
    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)?,
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    client_config.transport_config(Arc::new(transport_config));

    // Create QUIC endpoint bound to IPv6
    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0);
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);

    println!("Connecting to {} at {}", node_alt_name, node_addr);

    // Connect to the node
    let connection = endpoint
        .connect(node_addr, node_alt_name)?
        .await?;

    println!("Connected to {}", node_alt_name);

    // Open a bidirectional stream and send stream kind 0 (UP stream)
    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;
    send_stream.write_all(&[0]).await?;
    println!("Sent stream kind 0");

    // Immediately finish the send stream
    send_stream.finish()?;
    println!("Send stream closed");

    // Listen for incoming messages
    loop {
        // Read the 4-byte length prefix (little-endian u32)
        let mut len_buf = [0u8; 4];
        match recv_stream.read_exact(&mut len_buf).await {
            Ok(()) => {
                let len = u32::from_le_bytes(len_buf) as usize;
                // Ensure the length is reasonable to prevent allocation issues
                if len > 1024 * 1024 {
                    println!("Received unreasonably large message length: {}", len);
                    break;
                }
                // Read the message content
                let mut buffer = vec![0u8; len];
                match recv_stream.read_exact(&mut buffer).await {
                    Ok(()) => {
                        println!("Received message ({} bytes): {:?}", len, buffer);
                    }
                    Err(e) => {
                        println!("Error reading message content: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error reading message length: {}", e);
                break;
            }
        }
    }

    // Wait for connection closure
    connection.closed().await;
    println!("Connection closed");

    Ok(())
}




use base32::{Alphabet, encode};

const PUBKEY: [u8; 32] = [
    0x3b, 0x6a, 0x27, 0xbc, 0xce, 0xb6, 0xa4, 0x2d,
    0x62, 0xa3, 0xa8, 0xd0, 0x2a, 0x6f, 0x0d, 0x73,
    0x65, 0x32, 0x15, 0x77, 0x1d, 0xe2, 0x43, 0xa6,
    0x3a, 0xc0, 0x48, 0xa1, 0x8b, 0x59, 0xda, 0x29,
];

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
    println!("result hex: {}", hex::encode(result));
}