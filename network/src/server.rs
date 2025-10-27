use quinn::{ClientConfig, Endpoint, Incoming, SendStream, RecvStream, ServerConfig, Connection, StreamId, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};
use rustls::{Error as RustlsError, SignatureScheme, DistinguishedName};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use utils::common;
use core::num;
use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::error::Error;
use std::u32;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use jam_types::{*};
use codec::{BytesReader, Decode, Encode};
use crate::jamnp_types::Announcement;
use crate::net_utils::{parse_pem_private_key, parse_pem_certs};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const BLOCK_ANNOUNCEMENT: u8 = 0;
const BLOCK_REQUEST: u8 = 128;
const STATE_REQUEST: u8 = 129;
const TICKET_GENERATION: u8 = 131;
const TICKET_PROXY: u8 = 132;

struct TicketDistribution {
    epoch_index: TimeSlot,
    ticket: Ticket,
}

struct ConnectionInfo {
    connection: Connection,
    send_stream: SendStream,
    recv_stream: RecvStream,
    kind: u8,
}

struct BlockRequestInfo {
    author: ValidatorIndex,
    header_hash: OpaqueHash,
    direction: u8,
    num_blocks: u32,
}

struct StateRequestInfo {
    header_hash: OpaqueHash,
    keyvals: Vec<KeyValue>,
    max_size: u32,
}

enum Direction {
    AscendingExclusive = 0,
    DescendingInclusive = 1,
}

struct NetworkMessage {
    msg: Vec<u8>
}

impl NetworkMessage {
    fn new(kind: u8, payload: Vec<u8>) -> Vec<u8> {
        let len = payload.len() as u32;
        let len_bytes = len.to_le_bytes();
        let mut msg = Vec::with_capacity(5 + payload.len());
        msg.push(kind);
        msg.extend_from_slice(&len_bytes);
        msg.extend(payload);
        msg
    }

    fn prepend_kind_and_len(&mut self, kind: u8) {
        let len = self.msg.len() as u32;
        let len_bytes = len.to_le_bytes();
        let old_len = self.msg.len();
        self.msg.resize(old_len + 5, 0);
        self.msg.copy_within(0..old_len, 5);
        self.msg[0] = kind;
        self.msg[1..5].copy_from_slice(&len_bytes);
    }
}

pub async fn run_server() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000);

    let genesis_hash = "2bf11dc5";
    let alpn_protocol = format!("jamnp-s/0/{}", genesis_hash).into_bytes();

    let cert_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/certs/node0/cert.pem")?;
    let key_pem = std::fs::read("/home/bernar/workspace/vinwolf/network/src/certs/node0/key.pem")?;

    let certs: Vec<CertificateDer> = parse_pem_certs(&cert_pem)?;
    if certs.is_empty() {
        return Err("No valid certificates found in cert.pem".into());
    }

    let key_der = parse_pem_private_key(&key_pem)?;

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_client_cert_verifier(SkipClientVerification::new())
        .with_single_cert(certs, key_der)?;

    server_crypto.alpn_protocols = vec![alpn_protocol];

    let mut server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)?
    ));
    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    server_config.transport = Arc::new(transport_config);

    let endpoint = Endpoint::server(server_config, bind_addr)?;

    println!("Listening on {}", bind_addr);

    while let Some(conn) = endpoint.accept().await {
        
        println!("Incoming connection attempt from {}", conn.remote_address());
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    println!("New connection established from {}", connection.remote_address());
                    handle_connection(connection).await;
                }
                Err(e) => {
                    println!("Connection error: {}", e);
                }
            }
        });
    }

    endpoint.wait_idle().await;

    Ok(())
}


async fn handle_connection(connection: Connection) {

    println!("New connection established from {}", connection.remote_address());
    let conn_clone = connection.clone();
    // Wait for a new stream
    while let Ok((send_stream, mut recv_stream)) = connection.accept_bi().await {
        let conn_clone = conn_clone.clone(); // Clone for each spawn
        tokio::spawn(async move {
            let mut stream_kind_buf = [0u8; 1];
            if recv_stream.read_exact(&mut stream_kind_buf).await.is_ok() {
                println!("Received stream kind {:?}", stream_kind_buf);
                let conn_info = ConnectionInfo {
                    connection: conn_clone,
                    send_stream,
                    recv_stream,
                    kind: stream_kind_buf[0]
                };
                handle_stream(conn_info).await;
            }
        });
    }
}

async fn handle_stream(connection_info: ConnectionInfo) {

    match connection_info.kind {

        BLOCK_ANNOUNCEMENT => {
            block_announcement(connection_info).await;
        },
        BLOCK_REQUEST => {

        },
        STATE_REQUEST => {

        },
        TICKET_GENERATION => {
            println!("TICKET GENERATION");
            recv_ticket_distribution(connection_info).await;
        },
        TICKET_PROXY => {
            println!("TICKET PROXY");
            recv_ticket_distribution(connection_info).await;
        }, 
        _ => {
            println!("Unknown stream kind: {:?}", connection_info.kind);
        },
    }
}

async fn recv_ticket_distribution(connection_info: ConnectionInfo) {

    let mut send_stream = connection_info.send_stream;
    let mut recv_stream = connection_info.recv_stream;

    let mut len_msg = [0u8; 4];
    recv_stream.read_exact(&mut len_msg).await.unwrap();

    let len_msg = u32::from_le_bytes(len_msg) as usize;
    let mut buffer = vec![0u8; len_msg];

    recv_stream.read_exact(&mut buffer).await.unwrap();

    println!("ticket distribution msg recv: {}", utils::hex::encode(&buffer));
}

async fn state_request(header_hash: OpaqueHash, connection: Connection) -> GlobalState {

    let key_start = [0u8; 31];
    let key_end = [0xFFu8; 31];
    let max_size = u32::MAX;

    let payload = [header_hash.encode(), key_start.encode(), key_end.encode(), max_size.encode()].concat();
    let message = NetworkMessage::new(STATE_REQUEST, payload);

    let (mut send_stream, mut recv_stream) = connection.open_bi().await.unwrap();
    
    send_stream.write_all(&message).await.ok();
    send_stream.finish().unwrap();


    println!("State request sent for header: {}", utils::hex::encode(&header_hash));
    let mut state_response_len = [0u8; 4];
    
    recv_stream.read_exact(&mut state_response_len).await.unwrap();
    println!("response len: {:?}", state_response_len);
    let mut buffer = vec![0u8; u32::from_le_bytes(state_response_len) as usize];

    recv_stream.read_exact(&mut buffer).await.unwrap();
    println!("Boundaries received: {:?} bytes", buffer.len());
    //println!("payload: {:x?}", buffer);

    let mut state_response_len = [0u8; 4];
    recv_stream.read_exact(&mut state_response_len).await.unwrap();
    println!("response len: {:?}", state_response_len);
    let mut buffer = vec![0u8; u32::from_le_bytes(state_response_len) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();
    println!("Bytes state received: {:?}", buffer.len());
    //println!("State received: {:x?} bytes", buffer);
    drop(recv_stream);

    let mut reader = BytesReader::new(&buffer);

    let mut keyvals: Vec<KeyValue> = vec![];
    let mut c = 0;

    while reader.get_position() < u32::from_le_bytes(state_response_len) as usize {
        c+=1;
        keyvals.push(KeyValue::decode(&mut reader).unwrap());
    }

    let mut global_state = GlobalState::default();

    common::parse_state_keyvals(&keyvals, &mut global_state).unwrap();

    println!("Total keyvalues decoded: {c}");

    /*let mut reader = BytesReader::new(&buffer);
    let state_root = OpaqueHash::decode(&mut reader).unwrap();*/



    /*let mut reader = BytesReader::new(&buffer);
    let block = Block::decode(&mut reader).unwrap();*/

    return GlobalState::default();
} 

async fn block_request(request_info: BlockRequestInfo, connection: Connection) -> Block {

    let payload = [request_info.header_hash.encode(), vec![request_info.direction], request_info.num_blocks.to_le_bytes().to_vec()].concat();
    let message = NetworkMessage::new(BLOCK_REQUEST, payload);
    
    let (mut send_stream, mut recv_stream) = connection.open_bi().await.unwrap();

    send_stream.write_all(&message).await.ok();

    /*send_stream.write_all(&[BLOCK_REQUEST]).await.ok();
    let payload_len = payload.len() as u32;
    send_stream.write_all(&(payload_len.encode())).await.ok();
    send_stream.write_all(&payload).await.ok();*/
    
    send_stream.finish().unwrap();

    println!("Block request sent");
    let mut block_response_len = [0u8; 4];
    recv_stream.read_exact(&mut block_response_len).await.unwrap();
    let mut buffer = vec![0u8; u32::from_le_bytes(block_response_len) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();
    println!("Block received: {:?} bytes", buffer.len());
    let mut reader = BytesReader::new(&buffer);
    let block = Block::decode(&mut reader).unwrap();
    println!("Block: {:?}", block);
    return block;
}

async fn block_announcement(connection_info: ConnectionInfo) {

    let mut send_stream = connection_info.send_stream;
    let mut recv_stream = connection_info.recv_stream;

    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    let len_bytes = (handshake.len() as u32).to_le_bytes();
    /*send_stream.write_all(&len_bytes).await.ok();
    send_stream.write_all(&handshake).await.ok();*/
    send_stream.write_all(&([len_bytes.to_vec(), handshake].concat())).await.ok();
    println!("Sent handshake response");

    let mut len_handshake = [0u8; 4];
    recv_stream.read_exact(&mut len_handshake).await.unwrap();
    let mut buffer = vec![0u8; u32::from_le_bytes(len_handshake) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();
    println!("Handshake received: {:?} bytes", buffer.len());

    loop {
        let mut len_buf = [0u8; 4];
        match recv_stream.read_exact(&mut len_buf).await {
            Ok(()) => {
                let len = u32::from_le_bytes(len_buf) as usize;
                if len > 1024 * 1024 {
                    println!("Received unreasonably large message length: {}", len);
                    break;
                }
                let mut buffer = vec![0u8; len];
                match recv_stream.read_exact(&mut buffer).await {
                    Ok(()) => {
                        let mut reader = BytesReader::new(&buffer);
                        let announcement = Announcement::decode(&mut reader).unwrap();
                        let header_hash = sp_core::blake2_256(&announcement.header.encode());
                        println!("Import block {}", utils::print_hash!(header_hash));
                        let request_info = BlockRequestInfo {
                            author: announcement.header.unsigned.author_index,
                            header_hash: header_hash,
                            direction: 1,
                            num_blocks: 1
                        };
                        //tokio::spawn(state_request(announcement.last_block.header_hash, connection_info.connection.clone()));
                        //tokio::spawn(block_request(request_info, connection_info.connection.clone()));
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
}


#[derive(Debug)]
pub struct SkipClientVerification(Arc<CryptoProvider>);

impl SkipClientVerification {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(default_provider())))
    }
}

impl ClientCertVerifier for SkipClientVerification {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> std::result::Result<ClientCertVerified, rustls::Error> {
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

