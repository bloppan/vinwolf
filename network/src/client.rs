use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::pki_types::{CertificateDer};
use std::net::{IpAddr, Ipv6Addr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;

use codec::{Encode, Decode, BytesReader};
use jam_types::{*};
use crate::jamnp_types::{Handshake, Announcement};
use crate::net_utils::{parse_pem_private_key, parse_pem_certs, SkipServerVerification};
use utils::log;

pub async fn run_client(endpoint: Endpoint) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {

    /*rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;*/

    let node_alt_name = "ekwmt37xecoq6a7otkm4ux5gfmm4uwbat4bg5m223shckhaaxdpqa";
    let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40003);

    /*let genesis_hash = "2bf11dc5";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    let cert_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/certs/node0/cert.pem")?;
    let key_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/certs/node0/key.pem")?;

    let certs: Vec<CertificateDer> = parse_pem_certs(&cert_pem)?;
    if certs.is_empty() {
        return Err("No valid certificates found in cert.pem".into());
    }

    let key_der = parse_pem_private_key(&key_pem)?;

    let mut client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_client_auth_cert(certs, key_der)?;

    client_crypto.alpn_protocols = vec![alpn_protocol];

    let mut client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(client_crypto)?,
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    client_config.transport_config(Arc::new(transport_config));

    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 40000);
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);*/

    log::info!("Connecting to {} at {}", node_alt_name, node_addr);

    let connection = endpoint
        .connect(node_addr, node_alt_name)?
        .await?;

    log::info!("Connected to {}", node_alt_name);

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;
    send_stream.write_all(&[0]).await?;
    log::info!("Sent stream kind 0");

    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    let len_bytes = (handshake.len() as u32).to_le_bytes();
    send_stream.write_all(&len_bytes).await?;
    send_stream.write_all(&handshake).await?;
    log::info!("Sent handshake response");

    let mut len_buf = [0u8; 4];
    recv_stream.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut buffer = vec![0u8; len];
    recv_stream.read_exact(&mut buffer).await?;

    let mut reader = BytesReader::new(&buffer);
    let handshake = Handshake::decode(&mut reader).unwrap();
    log::info!("handshake: {:?}", handshake);

    loop {
        let mut len_buf = [0u8; 4];
        match recv_stream.read_exact(&mut len_buf).await {
            Ok(()) => {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len > 1024 * 1024 {
                    log::info!("Received unreasonably large message length: {}", len);
                    break;
                }
                let mut buffer = vec![0u8; len];
                match recv_stream.read_exact(&mut buffer).await {
                    Ok(()) => {
                        let mut reader = BytesReader::new(&buffer);
                        let announcement = Announcement::decode(&mut reader).unwrap();
                        log::info!("Received message ({} bytes): {:?}", len, announcement);
                        let curr_header_hash = sp_core::blake2_256(&announcement.header.encode());
                        log::info!("Curr header hash: {}", utils::hex::encode(&curr_header_hash));
                    }
                    Err(e) => {
                        log::error!("Error reading message content: {}", e);
                        break;
                    }
                }
            }
            Err(e) => {
                log::error!("Error reading message length: {}", e);
                break;
            }
        }
    }

    connection.closed().await;
    log::info!("Connection closed");

    Ok(())
}

