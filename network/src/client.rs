use codec::generic_codec::decode_from_bytes;
use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::pki_types::{CertificateDer};
use std::net::{IpAddr, Ipv6Addr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;

use codec::{Encode, Decode, BytesReader};
use crate::{dev_accounts, net_utils};
use crate::jamnp_types::{Handshake, Announcement};
use crate::message::{self, NetworkMessage, BLOCK_ANNOUNCEMENT};
use crate::net_utils::{parse_pem_private_key, parse_pem_certs, SkipServerVerification};
use jam_types::{*};
use utils::log;

pub async fn run_client(endpoint: Endpoint, client_index: ValidatorIndex) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {

    let dev_accounts = dev_accounts::parse_dev_accounts();
    let node_alt_name = dev_accounts[client_index as usize].dns_alt_name.clone();
    let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000 + client_index);

    log::info!("Connecting to {} at {}", node_alt_name, node_addr);

    let connection = endpoint
        .connect(node_addr, &node_alt_name)?
        .await?;

    log::info!("Connected to {}", node_alt_name);
    //return Ok(());

    let conn_clone = connection.clone();
    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    /*let connection_info = message::ConnectionInfo {
        connection: conn_clone,
        send_stream,
        recv_stream,
        kind: BLOCK_ANNOUNCEMENT
    };*/

    /*tokio::spawn(async move {
        message::block_announcement(connection_info).await.unwrap();
    });*/

    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    
    NetworkMessage::send_up(BLOCK_ANNOUNCEMENT, handshake, &mut send_stream).await.unwrap();
    let buffer = NetworkMessage::recv(&mut recv_stream).await.unwrap();
    let handshake = decode_from_bytes::<Handshake>(&buffer).unwrap();
    
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

