use crate::{dev_accounts, node_config};
use crate::jamnp_types::{Announcement, ConnectionError, Handshake, ImportedBlocks, NetworkError, StreamError, TicketDistributed};
use codec::{BytesReader, Decode, Encode};
use codec::generic_codec::decode_from_bytes;
use jam_types::{*};
use quinn::{SendStream, RecvStream, Connection};
use std::sync::{LazyLock, Mutex};
use std::u32;
use utils::{common, hex, log};

pub const BLOCK_ANNOUNCEMENT: u8 = 0;
pub const BLOCK_REQUEST: u8 = 128;
pub const STATE_REQUEST: u8 = 129;
pub const TICKET_GENERATION: u8 = 131;
pub const TICKET_PROXY: u8 = 132;

pub struct TicketDistribution {
    pub epoch_index: TimeSlot,
    pub ticket: Ticket,
}

pub struct ConnectionInfo {
    pub connection: Connection,
    pub send_stream: SendStream,
    pub recv_stream: RecvStream,
    pub kind: u8,
}

pub struct BlockRequestInfo {
    pub header_hash: OpaqueHash,
    pub direction: u8,
    pub num_blocks: u32,
}

pub struct StateRequestInfo {
    pub header_hash: OpaqueHash,
    pub keyvals: Vec<KeyValue>,
    pub max_size: u32,
}

pub enum Direction {
    AscendingExclusive = 0,
    DescendingInclusive = 1,
}

pub struct NetworkMessage {
    pub msg: Vec<u8>
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

    async fn recv(recv_stream: &mut RecvStream) -> Result<Vec<u8>, NetworkError> {

        let mut len_msg = [0u8; 4];
    
        if let Err(e) =  recv_stream.read_exact(&mut len_msg).await {
            log::error!("Failed to read the stream's length: {:?}", e);
            return Err(NetworkError::StreamError(StreamError::ReadStream));
        }

        let len_msg = u32::from_le_bytes(len_msg) as usize;
        let mut buffer = vec![0u8; len_msg];

        if let Err(e) = recv_stream.read_exact(&mut buffer).await {
            log::error!("Failed to read the stream. Msg len: {:?}. {:?}", len_msg, e);
            return Err(NetworkError::StreamError(StreamError::ReadStream));
        }

        return Ok(buffer);
    }

    async fn send(msg_kind: u8, payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        
        let message = NetworkMessage::new(msg_kind, payload);        
        
        if let Err(e) = send_stream.write_all(&message).await {
            log::error!("Failed to send stream: {:?}", e);
            return Err(NetworkError::StreamError(StreamError::WriteStream));
        }
        
        if let Err(e) = send_stream.finish() {
            log::error!("Failed to finish stream: {:?}", e);
            return Err(NetworkError::StreamError(StreamError::WriteStream));
        }

        return Ok(());
    }
}

pub async fn handle_stream(connection_info: ConnectionInfo) {

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
            recv_ticket_from_generator(connection_info).await.unwrap();
        },
        TICKET_PROXY => {
            log::debug!("Received ticket from proxy -> Include in a block");
            //recv_ticket_distribution(connection_info).await;
        },
        _ => {
            println!("Unknown stream kind: {:?}", connection_info.kind);
        },
    }
}

pub async fn recv_ticket_from_generator(connection_info: ConnectionInfo) -> Result<(), NetworkError> {

    let mut recv_stream = connection_info.recv_stream;
    let distributed_ticket_blob = NetworkMessage::recv(&mut recv_stream).await?;
    
    let distributed_ticket= decode_from_bytes::<TicketDistributed>(&distributed_ticket_blob).map_err(|e| {
        log::error!("Failed to decode distributed ticket: {:?}", e);
        NetworkError::ReadError(e)
    })?;

    log::debug!("Ticket: {:?}", distributed_ticket.ticket);

    tokio::spawn(async move { 
        broadcast_ticket_to_validators(distributed_ticket_blob).await; 
    });

    return Ok(());
}

pub async fn broadcast_ticket_to_validators(distributed_ticket_blob: Vec<u8>) {

    let proxy_ticket_msg = NetworkMessage::new(TICKET_PROXY, distributed_ticket_blob);

    let validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.curr_validators.list.clone()
    };

    let this_node = node_config::get_account_id();

    for (i, validator) in validators.iter().enumerate() {
        
        if i == this_node as usize {
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
        send_stream.write_all(&proxy_ticket_msg).await.ok();
        send_stream.finish().unwrap();
    }
}

async fn recv_ticket_distribution(connection_info: ConnectionInfo) -> Result<(), NetworkError> {

    let mut recv_stream = connection_info.recv_stream;
    let distributed_ticket = NetworkMessage::recv(&mut recv_stream).await?;
    log::debug!("ticket distribution msg recv: {}", utils::hex::encode(&distributed_ticket));
    
    let state = state_handler::get_global_state().lock().unwrap();
    log::debug!("Current validators: {:x?}", state.curr_validators.list);
    log::debug!("Next validators: {:x?}", state.next_validators.list);

    Ok(())
}

async fn state_request(header_hash: OpaqueHash, connection: Connection) -> Result<GlobalState, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await.map_err(|e| {
        log::error!("Failed to open bidirectional stream: {:?}", e);
        NetworkError::ConnectionError(ConnectionError::OpenBidirectionalStream)
    })?;

    let key_start = [0u8; 31];
    let key_end = [0xFFu8; 31];
    let max_size = u32::MAX;
    
    let payload = [header_hash.encode(), key_start.encode(), key_end.encode(), max_size.encode()].concat();

    NetworkMessage::send(STATE_REQUEST, payload, &mut send_stream).await?;
    log::debug!("State request sent to address: {:?} for header: {}", connection.remote_address(), utils::hex::encode(&header_hash));
    
    let _boundary_nodes = NetworkMessage::recv(&mut recv_stream).await?;
    let state_keyvals_blob = NetworkMessage::recv(&mut recv_stream).await?;
    
    drop(recv_stream);

    let mut reader = BytesReader::new(&state_keyvals_blob);
    let mut keyvals: Vec<KeyValue> = vec![];
    let mut c: usize = 0;

    while reader.get_position() < state_keyvals_blob.len() {

        c = c.saturating_add(1);

        let keyval = KeyValue::decode(&mut reader).map_err(|e| {
            log::error!("Failed to decode keyvalue {c}: {:?}", e);
            NetworkError::ReadError(e)
        })?;

        keyvals.push(keyval);
    }

    let mut global_state = GlobalState::default();
    
    if let Err(e) = common::parse_state_keyvals(&keyvals, &mut global_state) {
        log::error!("Failed to parse state keyvals: {:?}", e);
        return Err(NetworkError::ReadError(e));
    }
    
    log::debug!("Total keyvalues decoded: {c}");

    return Ok(global_state);
} 

async fn block_request(request_info: BlockRequestInfo, connection: Connection) -> Result<Vec<Block>, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await.map_err(|e| {
        log::error!("Failed to open bidirectional stream: {:?}", e);
        NetworkError::ConnectionError(ConnectionError::OpenBidirectionalStream)
    })?;

    let payload = [request_info.header_hash.encode(), vec![request_info.direction], request_info.num_blocks.to_le_bytes().to_vec()].concat();
    NetworkMessage::send(BLOCK_REQUEST, payload, &mut send_stream).await?;
    log::debug!("Block request sent from block {} num_blocks: {:?}", utils::hex::encode(&request_info.header_hash), request_info.num_blocks);

    let blocks_blob = NetworkMessage::recv(&mut recv_stream).await?;
    log::debug!("Blocks received: {:?} bytes", blocks_blob.len());
    
    let mut blocks = vec![];
    let mut reader = BytesReader::new(&blocks_blob);
    
    for i in 0..request_info.num_blocks {
        
        let block = Block::decode(&mut reader).map_err(|e| {
            log::error!("Failed to decode block {i}: {:?}", e);
            NetworkError::ReadError(e)
        })?;

        log::debug!("Slot: {:?} Block: {:x?}", block.header.unsigned.slot, block);
        blocks.push(block);
    }    

    return Ok(blocks);
}

async fn sync_blocks(imported_blocks_recv: ImportedBlocks, connection: Connection) -> Result<(), NetworkError> {

    if is_synced(&imported_blocks_recv) {
        log::debug!("The sync from address: {:?} is already done", connection.remote_address());
        return Ok(());
    }

    log::debug!("Syncing blocks from address: {:?}", connection.remote_address());

    let imported_blocks_stored = 
    {
        IMPORTED_BLOCKS.lock().unwrap().clone()
    };

    let last_global_finalized_state = state_request(imported_blocks_stored.last_finalized_block.header_hash, connection.clone()).await?;
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
    let mut blocks = block_request(request_info, connection.clone()).await?;
    blocks.sort_by_key(|block| block.header.unsigned.slot);

    for block in blocks.iter() {
        log::debug!("SYNC process block {}", utils::hex::encode(&sp_core::blake2_256(&block.header.encode())));
        match state_controller::stf(block) {
            Ok(_) => { log::debug!("processed successfully"); },
            Err(e) => { log::error!("error processing block: {:?}", e); },
        }
    }

    return Ok(());
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

pub async fn block_announcement(connection_info: ConnectionInfo) -> Result<(), NetworkError> {

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

    if let Err(e) = sync_blocks(imported_blocks, connection_info.connection.clone()).await {
        log::error!("Failed to sync blocks: {:?}", e);
        return Err(e);
    }
    
    loop {

        let announcement_blob = NetworkMessage::recv(&mut recv_stream).await?;

        let announcement = match decode_from_bytes::<Announcement>(&announcement_blob) {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to decode announcement: {:?}", e);
                continue;
            }
        };

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

        let block = block_request(request_info, connection_info.connection.clone()).await.unwrap();
        log::debug!("process block in loop {}", utils::hex::encode(&sp_core::blake2_256(&block[0].header.encode())));

        match state_controller::stf(&block[0]) {
            Ok(_) => { log::debug!("block successfully processed"); }
            Err(e) => { log::error!("error processing block: {:?}", e); }
        }

        let state = state_handler::get_global_state().lock().unwrap().clone();
        //tokio::spawn(state_request(announcement.last_finalized_block.header_hash, connection_info.connection.clone()));
        //tokio::spawn(block_request(request_info, connection_info.connection.clone()));
    }
}

