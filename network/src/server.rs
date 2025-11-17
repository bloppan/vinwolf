use quinn::{ClientConfig, Endpoint, Incoming, SendStream, RecvStream, ServerConfig, Connection, StreamId, TransportConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime, PrivatePkcs8KeyDer};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};
use rustls::{Error as RustlsError, SignatureScheme, DistinguishedName};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;

use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::id;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use std::error::Error;
use std::u32;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use utils::{common, hex, log};
use jam_types::{*};
use codec::{BytesReader, Decode, Encode};
use crate::dev_accounts;
use crate::jamnp_types::{Announcement, Handshake, ImportedBlocks};
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

    log::debug!("Listening on {}", bind_addr);

    while let Some(conn) = endpoint.accept().await {
        log::debug!("Incoming connection attempt from {}", conn.remote_address());
        tokio::spawn(async move {
            match conn.await {
                Ok(connection) => {
                    let id_account = connection.remote_address().port() & 0xF;
                    let dev_accounts = dev_accounts::parse_dev_accounts();
                    log::debug!("New connection established from {} bandersnatch public: {}", connection.remote_address(), hex::encode(&dev_accounts[id_account as usize].bandersnatch_public));
                    dev_accounts::add_dev_account(dev_accounts[id_account as usize].bandersnatch_public, connection.clone());
                    handle_connection(connection).await;
                }
                Err(e) => {
                    log::error!("Connection error: {}", e);
                }
            }
        });
    }

    endpoint.wait_idle().await;

    Ok(())
}


async fn handle_connection(connection: Connection) {

    log::debug!("New connection established from {}", connection.remote_address());
    let conn_clone = connection.clone();
    // Wait for a new stream
    while let Ok((send_stream, mut recv_stream)) = connection.accept_bi().await {
        let conn_clone = conn_clone.clone(); // Clone for each spawn
        tokio::spawn(async move {
            let mut stream_kind_buf = [0u8; 1];
            if recv_stream.read_exact(&mut stream_kind_buf).await.is_ok() {
                log::debug!("Received stream kind {:?}", stream_kind_buf);
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
            log::debug!("Generated ticket received -> Send to all current validators");
            proxy_ticket_received(connection_info).await;
        },
        TICKET_PROXY => {
            log::debug!("TICKET PROXY");
            //recv_ticket_distribution(connection_info).await;
        }, 
        _ => {
            println!("Unknown stream kind: {:?}", connection_info.kind);
        },
    }
}

async fn proxy_ticket_received(connection_info: ConnectionInfo) {

    let mut recv_stream = connection_info.recv_stream;
    let mut len_msg = [0u8; 4];
    recv_stream.read_exact(&mut len_msg).await.unwrap();

    let len_msg = u32::from_le_bytes(len_msg) as usize;
    let mut buffer = vec![0u8; len_msg];
    recv_stream.read_exact(&mut buffer).await.unwrap();

    let mut reader = BytesReader::new(&buffer);
    let _epoch_index = TimeSlot::decode(&mut reader).unwrap();
    let ticket = Ticket::decode(&mut reader).unwrap();
    log::debug!("Ticket: {:?}", ticket);
    log::debug!("Curr pos: {:?} end: {:?}", reader.get_position(), buffer.len());
    let message = NetworkMessage::new(TICKET_PROXY, buffer);

    tokio::spawn(async move { proxy_ticket_to_validators(message).await; });
}

async fn proxy_ticket_to_validators(message: Vec<u8>) {
    let validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.curr_validators.list.clone()
    };

    for (i, validator) in validators.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let connection = match dev_accounts::get_dev_account_connection(&validator.bandersnatch) {
            Some(conn) => conn,
            None => {
                log::debug!("Error getting dev account connection for validator: {i}");
                continue;
            }
        };
        log::debug!("Proxy ticket to {:?}", connection.remote_address());
        let (mut send_stream, mut _recv_stream) = connection.open_bi().await.unwrap();
        send_stream.write_all(&message).await.ok();
        send_stream.finish().unwrap();
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

    log::debug!("ticket distribution msg recv: {}", utils::hex::encode(&buffer));
    

    let state = state_handler::get_global_state().lock().unwrap();
    log::debug!("Current validators: {:x?}", state.curr_validators.list);
    log::debug!("Next validators: {:x?}", state.next_validators.list);
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

    log::debug!("State request sent to address: {:?} for header: {}", connection.remote_address(), utils::hex::encode(&header_hash));
    let mut state_response_len = [0u8; 4];
    
    recv_stream.read_exact(&mut state_response_len).await.unwrap();
    log::debug!("response len: {:?}", state_response_len);
    let mut buffer = vec![0u8; u32::from_le_bytes(state_response_len) as usize];

    recv_stream.read_exact(&mut buffer).await.unwrap();
    //log::debug!("Boundaries received: {:x?} bytes", buffer);
    log::debug!("Total {:?} nodes", buffer.len() / 64);
    //println!("payload: {:x?}", buffer);

    let mut state_response_len = [0u8; 4];
    recv_stream.read_exact(&mut state_response_len).await.unwrap();
    log::debug!("response len: {:?}", state_response_len);
    let mut buffer = vec![0u8; u32::from_le_bytes(state_response_len) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();
    log::debug!("Bytes state received: {:?}", buffer.len());

    //log::debug!("State received: {:x?} bytes", buffer);
    
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

    log::debug!("Total keyvalues decoded: {c}");

    /*let mut reader = BytesReader::new(&buffer);
    let state_root = OpaqueHash::decode(&mut reader).unwrap();*/

    /*let mut reader = BytesReader::new(&buffer);
    let block = Block::decode(&mut reader).unwrap();*/

    return global_state;
} 

async fn block_request(request_info: BlockRequestInfo, connection: Connection) -> Vec<Block> {

    let payload = [request_info.header_hash.encode(), vec![request_info.direction], request_info.num_blocks.to_le_bytes().to_vec()].concat();
    let message = NetworkMessage::new(BLOCK_REQUEST, payload);
    
    let (mut send_stream, mut recv_stream) = connection.open_bi().await.unwrap();

    send_stream.write_all(&message).await.ok();    
    send_stream.finish().unwrap();

    log::debug!("Block request sent from block {} num_blocks: {:?}", utils::hex::encode(&request_info.header_hash), request_info.num_blocks);

    let mut block_response_len = [0u8; 4];
    recv_stream.read_exact(&mut block_response_len).await.unwrap();
    let mut buffer = vec![0u8; u32::from_le_bytes(block_response_len) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();

    log::debug!("Block received: {:?} bytes", buffer.len());
    
    let mut blocks = vec![];
    let mut reader = BytesReader::new(&buffer);
    
    for _ in 0..request_info.num_blocks {
        let block = Block::decode(&mut reader).unwrap();
        log::debug!("Slot: {:?} Block: {:x?}", block.header.unsigned.slot, block);
        blocks.push(block);
    }    

    return blocks;
}



async fn sync_blocks(imported_blocks_recv: ImportedBlocks, connection: Connection) {

    if is_synced(&imported_blocks_recv) {
        log::debug!("The sync from address: {:?} is already done", connection.remote_address());
        return;
    }

    log::debug!("Syncing blocks from address: {:?}", connection.remote_address());

    let imported_blocks_stored = 
    {
        IMPORTED_BLOCKS.lock().unwrap().clone()
    };

    /*let last_global_finalized_state = state_request(imported_blocks_stored.leafs[0].header_hash, connection.clone()).await;
    block::header::set_parent_header(imported_blocks_stored.leafs[0].header_hash);
    log::debug!("LEAF Synced parent header: {}", utils::hex::encode(&imported_blocks_stored.leafs[0].header_hash));
    // Calc state root
    let state_root = utils::trie::merkle_state(&utils::serialization::serialize(&last_global_finalized_state).map);
    state_handler::set_state_root(state_root);
    log::debug!("LEAF Synced state root: {}", hex::encode(&state_root));
    // Initialize the verifiers 
    safrole::verifier::init_all(&last_global_finalized_state);
    // Set global state
    state_handler::set_global_state(last_global_finalized_state);
    let time = state_handler::time::get();
    log::debug!("LEAF Time set: {time}");*/

    let last_global_finalized_state = state_request(imported_blocks_stored.last_finalized_block.header_hash, connection.clone()).await;
    block::header::set_parent_header(imported_blocks_stored.last_finalized_block.header_hash);
    log::debug!("Synced parent header: {}", utils::hex::encode(&imported_blocks_stored.last_finalized_block.header_hash));
    // Calc state root
    let state_root = utils::trie::merkle_state(&utils::serialization::serialize(&last_global_finalized_state).map);
    state_handler::set_state_root(state_root);
    log::debug!("Synced state root: {}", hex::encode(&state_root));
    // Initialize the verifiers 
    safrole::verifier::init_all(&last_global_finalized_state);
    // Set global state
    state_handler::set_global_state(last_global_finalized_state);
    let time = state_handler::time::get();
    log::debug!("Time set: {time}");

    let request_info = BlockRequestInfo {
                            header_hash: imported_blocks_stored.leafs[0].header_hash,
                            direction: 1,
                            num_blocks: imported_blocks_stored.leafs[0].slot - imported_blocks_stored.last_finalized_block.slot
                        };

    log::debug!("Request blocks from slot {:?} in order to sync", imported_blocks_recv.leafs[0].slot);
    let mut blocks = block_request(request_info, connection.clone()).await;
    blocks.sort_by_key(|block| block.header.unsigned.slot);

    for block in blocks.iter() {
        log::debug!("SYNC process block {}", utils::hex::encode(&sp_core::blake2_256(&block.header.encode())));
        match state_controller::stf(block) {
            Ok(_) => { log::debug!("processed successfully"); },
            Err(e) => { log::error!("error processing block: {:?}", e); },
        }
    }
}

static IMPORTED_BLOCKS: LazyLock<Mutex<ImportedBlocks>> = LazyLock::new(|| { Mutex::new(ImportedBlocks::default()) });

fn is_synced(imported_blocks: &ImportedBlocks) -> bool {

    let mut imported_blocks_stored = IMPORTED_BLOCKS.lock().unwrap();

    if imported_blocks_stored.clone() == *imported_blocks {
        return true;
    }

    *imported_blocks_stored = imported_blocks.clone();

    return false;
}

static LAST_ANNOUNCEMENT: LazyLock<Mutex<Announcement>> = LazyLock::new(|| { Mutex::new(Announcement::default()) });

fn is_new(announcement: &Announcement) -> bool {

    let mut last_announcement_stored = LAST_ANNOUNCEMENT.lock().unwrap();
    
    if last_announcement_stored.header.unsigned.slot > announcement.header.unsigned.slot {
        return false;
    } 
    
    if last_announcement_stored.header.unsigned.slot == announcement.header.unsigned.slot 
    && sp_core::blake2_256(&last_announcement_stored.encode()) == sp_core::blake2_256(&announcement.encode()) {
        return false;
    }
    
    *last_announcement_stored = announcement.clone();

    return true;
}

async fn block_announcement(connection_info: ConnectionInfo) {

    let mut send_stream = connection_info.send_stream;
    let mut recv_stream = connection_info.recv_stream;

    let handshake = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
    let len_bytes = (handshake.len() as u32).to_le_bytes();
    /*send_stream.write_all(&len_bytes).await.ok();
    send_stream.write_all(&handshake).await.ok();*/
    send_stream.write_all(&([len_bytes.to_vec(), handshake].concat())).await.ok();
    log::debug!("Sent handshake response");
 
    let mut len_handshake = [0u8; 4];
    recv_stream.read_exact(&mut len_handshake).await.unwrap();
    let mut buffer = vec![0u8; u32::from_le_bytes(len_handshake) as usize];
    recv_stream.read_exact(&mut buffer).await.unwrap();
    log::debug!("Handshake received: {:?} bytes", buffer.len());

    let mut reader = BytesReader::new(&buffer);
    let handshake = Handshake::decode(&mut reader).unwrap();
    log::debug!("Last finalized block: {} slot: {:?}", utils::hex::encode(&handshake.last_finalized_block.header_hash), handshake.last_finalized_block.slot);
    log::debug!("Leafs: {} Slots: {:?}"
    , handshake.leafs.iter().map(|leaf| utils::hex::encode(&leaf.header_hash)).collect::<Vec<_>>().join(", "),  handshake.leafs.iter().map(|leaf| leaf.slot).collect::<Vec<TimeSlot>>());

    let imported_blocks = ImportedBlocks {
        last_finalized_block: handshake.last_finalized_block,
        leafs: handshake.leafs
    };

    sync_blocks(imported_blocks, connection_info.connection.clone()).await;
    
    loop {
        let mut len_buf = [0u8; 4];
        match recv_stream.read_exact(&mut len_buf).await {
            Ok(()) => {

                let mut buffer = vec![0u8; u32::from_le_bytes(len_buf) as usize];

                match recv_stream.read_exact(&mut buffer).await {
                    Ok(()) => {
                        let mut reader = BytesReader::new(&buffer);
                        let announcement = Announcement::decode(&mut reader).unwrap();

                        if !is_new(&announcement) {
                            continue;
                        }

                        let header_hash = sp_core::blake2_256(&announcement.header.encode());
                        log::debug!("Import block {} parent {}", utils::hex::encode(&header_hash), utils::hex::encode(&announcement.header.unsigned.parent));
                        let request_info = BlockRequestInfo {
                            header_hash,
                            direction: 1,
                            num_blocks: 1
                        };

                        let block = block_request(request_info, connection_info.connection.clone()).await;
                        log::debug!("process block in loop {}", utils::hex::encode(&sp_core::blake2_256(&block[0].header.encode())));

                        match state_controller::stf(&block[0]) {
                            Ok(_) => { log::debug!("block successfully processed"); }
                            Err(e) => { log::error!("error processing block: {:?}", e); }
                        }

                        let state = state_handler::get_global_state().lock().unwrap().clone();
                        let seal = block::header::create_seal(
                            &state.safrole,
                            &state.entropy, 
                            &state.curr_validators, 
                            &block[0].header.unsigned);
                        log::info!("Tickets or keys: {:?}", state.safrole.seal);
                        log::info!("Block header seal: {}", hex::encode(&block[0].header.seal));
                        log::info!("Calculated seal: {}", hex::encode(&seal));
                        //tokio::spawn(state_request(announcement.last_finalized_block.header_hash, connection_info.connection.clone()));
                        //tokio::spawn(block_request(request_info, connection_info.connection.clone()));
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

