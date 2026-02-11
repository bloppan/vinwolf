use crate::{dev_accounts, node_config};
use crate::jamnp_types::{StreamKind, Announcement, Handshake, ImportedBlocks, NetworkError, TicketDistributed};
use codec::{BytesReader, Decode, Encode};
use codec::generic_codec::decode_from_bytes;
use jam_types::{*};
use quinn::{SendStream, RecvStream, Connection};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{LazyLock, Mutex, OnceLock};
use tokio::sync::{mpsc, oneshot};
use tools::{hex, log};

pub const BLOCK_ANNOUNCEMENT: StreamKind = 0;
pub const BLOCK_REQUEST: StreamKind = 128;
pub const STATE_REQUEST: StreamKind = 129;
pub const TICKET_GENERATION: StreamKind = 131;
pub const TICKET_PROXY: StreamKind = 132;

static BLOCK_STORE: LazyLock<Mutex<HashMap<OpaqueHash, Block>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn store_block(block: &Block) {
    let header_hash = sp_core::blake2_256(&block.header.encode());
    log::debug!("Storing block {}", hex::encode(&header_hash));
    BLOCK_STORE.lock().unwrap().insert(header_hash, block.clone());
}

pub fn get_block(header_hash: &OpaqueHash) -> Option<Block> {
    BLOCK_STORE.lock().unwrap().get(header_hash).cloned()
}

// --- Block processing queue ---

pub struct QueuedBlock {
    pub block: Block,
    pub result_tx: Option<oneshot::Sender<Result<(), ProcessError>>>,
}

static BLOCK_QUEUE_TX: OnceLock<mpsc::Sender<QueuedBlock>> = OnceLock::new();

pub fn init_block_queue(buffer: usize) -> mpsc::Receiver<QueuedBlock> {
    let (tx, rx) = mpsc::channel(buffer);
    BLOCK_QUEUE_TX.set(tx).expect("init_block_queue called more than once");
    rx
}

pub async fn enqueue_block(block: Block) {
    let tx = BLOCK_QUEUE_TX.get().expect("block queue not initialized");
    let queued = QueuedBlock { block, result_tx: None };
    if let Err(e) = tx.send(queued).await {
        log::error!("Failed to enqueue block: {:?}", e);
    }
}

pub async fn enqueue_block_and_wait(block: Block) -> Result<(), ProcessError> {
    let tx = BLOCK_QUEUE_TX.get().expect("block queue not initialized");
    let (result_tx, result_rx) = oneshot::channel();
    let queued = QueuedBlock { block, result_tx: Some(result_tx) };
    tx.send(queued).await.map_err(|_| {
        log::error!("Failed to enqueue block for processing");
        ProcessError::HeaderError(HeaderErrorCode::BadParentHeader)
    })?;
    result_rx.await.unwrap_or_else(|_| {
        log::error!("Block queue consumer dropped without sending result");
        Err(ProcessError::HeaderError(HeaderErrorCode::BadParentHeader))
    })
}

pub async fn run_block_queue(mut rx: mpsc::Receiver<QueuedBlock>) {
    loop {
        // Wait for at least one block
        let first = match rx.recv().await {
            Some(qb) => qb,
            None => {
                log::info!("Block queue channel closed, consumer exiting");
                return;
            }
        };

        // Drain any additional queued blocks
        let mut pending = vec![first];
        while let Ok(qb) = rx.try_recv() {
            pending.push(qb);
        }

        // Sort by slot using BTreeMap
        let mut by_slot: BTreeMap<TimeSlot, Vec<QueuedBlock>> = BTreeMap::new();
        for qb in pending {
            let slot = qb.block.header.unsigned.slot;
            by_slot.entry(slot).or_default().push(qb);
        }

        // Process in ascending slot order
        for (slot, blocks) in by_slot {
            for qb in blocks {
                log::debug!("Queue: processing block slot={}", slot);
                store_block(&qb.block);
                let result = state_controller::stf(&qb.block);
                match &result {
                    Ok(_) => log::debug!("Queue: block slot={} processed successfully", slot),
                    Err(e) => log::error!("Queue: error processing block slot={}: {:?}", slot, e),
                }
                if let Some(tx) = qb.result_tx {
                    let _ = tx.send(result);
                }
            }
        }
    }
}

static TICKET_POOL: LazyLock<Mutex<Vec<Ticket>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn clear_ticket_pool() {
    TICKET_POOL.lock().unwrap().clear();
}

pub fn store_ticket(ticket: Ticket) {
    log::debug!("Storing ticket attempt={}", ticket.attempt);
    TICKET_POOL.lock().unwrap().push(ticket);
}

pub fn take_tickets_for_block(slot: TimeSlot) -> Vec<Ticket> {
    use constants::node::{EPOCH_LENGTH, MAX_TICKETS_PER_EXTRINSIC, TICKET_SUBMISSION_ENDS};

    let slot_in_epoch = slot % EPOCH_LENGTH as TimeSlot;
    if slot_in_epoch >= TICKET_SUBMISSION_ENDS as TimeSlot {
        return vec![];
    }

    let (safrole_state, entropy_state) = {
        let state = state_handler::get_global_state().lock().unwrap();
        (state.safrole.clone(), state.entropy.clone())
    };

    let existing_ids: Vec<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|t| t.id).collect();

    let verifier = safrole::verifier::get(ValidatorSet::Next);
    let fixed_input_data: Vec<u8> = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode()].concat();

    let mut pool = TICKET_POOL.lock().unwrap();

    let mut valid_tickets: Vec<(Ticket, OpaqueHash)> = Vec::new();

    pool.retain(|ticket| {
        let vrf_input = [&fixed_input_data[..], &ticket.attempt.encode()].concat();
        match verifier.ring_vrf_verify(&vrf_input, &[], &ticket.signature) {
            Ok(id) => {
                if existing_ids.contains(&id) {
                    log::debug!("Ticket already in accumulator, discarding");
                    false
                } else {
                    valid_tickets.push((ticket.clone(), id));
                    false
                }
            }
            Err(_) => {
                log::error!("Invalid ticket proof, discarding");
                false
            }
        }
    });

    valid_tickets.sort_by(|a, b| a.1.cmp(&b.1));
    let candidates: Vec<(Ticket, OpaqueHash)> = valid_tickets.into_iter()
        .take(MAX_TICKETS_PER_EXTRINSIC)
        .collect();

    // Simulate insertion into the accumulator to discard tickets that would be dropped
    let candidate_ids: Vec<OpaqueHash> = candidates.iter().map(|(_, id)| *id).collect();
    let mut simulated_acc: Vec<OpaqueHash> = existing_ids;
    simulated_acc.extend(&candidate_ids);
    simulated_acc.sort();

    let selected: Vec<Ticket> = if simulated_acc.len() > EPOCH_LENGTH {
        let survivors: HashSet<OpaqueHash> = simulated_acc[..EPOCH_LENGTH].iter().cloned().collect();
        candidates.into_iter()
            .filter(|(_, id)| survivors.contains(id))
            .map(|(ticket, _)| ticket)
            .collect()
    } else {
        candidates.into_iter()
            .map(|(ticket, _)| ticket)
            .collect()
    };

    log::debug!("Selected {} tickets for block at slot {}", selected.len(), slot);

    selected
}

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
    pub fn new(kind: u8, payload: Vec<u8>) -> Vec<u8> {
        let len = payload.len() as u32;
        let len_bytes = len.to_le_bytes();
        let mut msg = Vec::with_capacity(5 + payload.len());
        msg.push(kind);
        msg.extend_from_slice(&len_bytes);
        msg.extend(payload);
        msg
    }

    pub async fn recv(recv_stream: &mut RecvStream) -> Result<Vec<u8>, NetworkError> {
        let mut len_msg = [0u8; 4];
        recv_stream.read_exact(&mut len_msg).await?;
        let len_msg = u32::from_le_bytes(len_msg) as usize;
        let mut buffer = vec![0u8; len_msg];
        recv_stream.read_exact(&mut buffer).await?;
        Ok(buffer)
    }

    pub async fn send(msg_kind: u8, payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(msg_kind, payload);
        send_stream.write_all(&message).await?;
        send_stream.finish()?;
        Ok(())
    }

    pub async fn send_up(msg_kind: u8, payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let message = NetworkMessage::new(msg_kind, payload);
        send_stream.write_all(&message).await?;
        Ok(())
    }

    /// Sends a length-prefixed message without kind byte, then finishes the stream.
    /// Use for CE stream responses where the kind byte was already sent at stream open.
    pub async fn reply(payload: Vec<u8>, send_stream: &mut SendStream) -> Result<(), NetworkError> {
        let len_bytes = (payload.len() as u32).to_le_bytes();
        send_stream.write_all(&len_bytes).await?;
        send_stream.write_all(&payload).await?;
        send_stream.finish()?;
        Ok(())
    }
}

pub async fn send_ticket_to_proxy(distributed_ticket_blob: Vec<u8>, proxy_bandersnatch: &BandersnatchPublic) {

    let connection = match dev_accounts::get_dev_account_connection(proxy_bandersnatch) {
        Some(conn) => conn,
        None => {
            log::error!("No connection to proxy validator {}", hex::encode(proxy_bandersnatch));
            return;
        }
    };

    let (mut send_stream, _recv_stream) = match connection.open_bi().await {
        Ok(streams) => streams,
        Err(e) => {
            log::error!("Failed to open stream to proxy validator: {:?}", e);
            return;
        }
    };

    log::info!("Sending ticket to proxy validator at {:?} (CE 131)", connection.remote_address());
    if let Err(e) = NetworkMessage::send(TICKET_GENERATION, distributed_ticket_blob, &mut send_stream).await {
        log::error!("Failed to send ticket to proxy: {:?}", e);
    }
}

pub async fn recv_ticket_from_generator(mut recv_stream: RecvStream) -> Result<(), NetworkError> {

    let distributed_ticket_blob = NetworkMessage::recv(&mut recv_stream).await?;

    let distributed_ticket = decode_from_bytes::<TicketDistributed>(&distributed_ticket_blob).map_err(|e| {
        log::error!("Failed to decode distributed ticket: {:?}", e);
        NetworkError::Decode(e)
    })?;

    log::debug!("Ticket received (CE 131): attempt={} epoch={}",
        distributed_ticket.ticket.attempt, distributed_ticket.epoch);

    // Verify that we are the correct proxy for this ticket
    let entropy_state = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.entropy.clone()
    };

    let verifier = safrole::verifier::get(ValidatorSet::Next);
    let fixed_input_data: Vec<u8> = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode()].concat();
    let vrf_input = [&fixed_input_data[..], &distributed_ticket.ticket.attempt.encode()].concat();

    let ticket_id = match verifier.ring_vrf_verify(&vrf_input, &[], &distributed_ticket.ticket.signature) {
        Ok(id) => id,
        Err(_) => {
            log::error!("Invalid ticket proof received on CE 131, discarding");
            return Ok(());
        }
    };

    let proxy_index = {
        let last_4 = &ticket_id[28..32];
        let val = u32::from_be_bytes(last_4.try_into().unwrap());
        (val as usize) % constants::node::VALIDATORS_COUNT
    };

    let this_node = node_config::get_account_id() as usize;
    if proxy_index != this_node {
        log::error!("We are not the correct proxy for this ticket (proxy={}, we={}), discarding", proxy_index, this_node);
        return Ok(());
    }

    log::info!("We are the correct proxy for ticket attempt={}, forwarding to all validators (CE 132)",
        distributed_ticket.ticket.attempt);

    store_ticket(distributed_ticket.ticket);

    tokio::spawn(async move {
        broadcast_ticket_to_validators(distributed_ticket_blob).await;
    });

    Ok(())
}

pub async fn broadcast_ticket_to_validators(distributed_ticket_blob: Vec<u8>) {

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
                log::error!("Getting dev account connection for validator: {i}");
                continue;
            }
        };

        let (mut send_stream, _recv_stream) = match connection.open_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                log::error!("Failed to open stream to validator {i}: {:?}", e);
                continue;
            }
        };

        log::info!("Proxy ticket to validator {i} at {:?}", connection.remote_address());
        if let Err(e) = NetworkMessage::send(TICKET_PROXY, distributed_ticket_blob.clone(), &mut send_stream).await {
            log::error!("Failed to send ticket to validator {i}: {:?}", e);
        }
    }
}

pub async fn recv_ticket_distribution(mut recv_stream: RecvStream) -> Result<(), NetworkError> {

    let distributed_ticket_blob = NetworkMessage::recv(&mut recv_stream).await?;

    let distributed_ticket = decode_from_bytes::<TicketDistributed>(&distributed_ticket_blob).map_err(|e| {
        log::error!("Failed to decode distributed ticket: {:?}", e);
        NetworkError::Decode(e)
    })?;

    log::info!("Ticket distribution received: epoch={} attempt={}", distributed_ticket.epoch, distributed_ticket.ticket.attempt);
    store_ticket(distributed_ticket.ticket);

    Ok(())
}

async fn state_request(header_hash: OpaqueHash, connection: Connection) -> Result<GlobalState, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    let key_start = [0u8; 31];
    let key_end = [0xFFu8; 31];
    let max_size = u32::MAX;
    
    let payload = [header_hash.encode(), key_start.encode(), key_end.encode(), max_size.encode()].concat();

    NetworkMessage::send(STATE_REQUEST, payload, &mut send_stream).await?;
    log::debug!("State request sent to address: {:?} for header: {}", connection.remote_address(), hex::encode(&header_hash));
    
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
            NetworkError::Decode(e)
        })?;

        keyvals.push(keyval);
    }

    let mut global_state = GlobalState::default();
    
    misc::parse_state_keyvals(&keyvals, &mut global_state)?;
    
    // Initialize the verifiers 
    safrole::verifier::init_all(&global_state);
    
    log::debug!("Total keyvalues decoded: {c}");

    Ok(global_state)
} 

pub async fn handle_block_request(mut send_stream: SendStream, mut recv_stream: RecvStream) -> Result<(), NetworkError> {

    let request_blob = NetworkMessage::recv(&mut recv_stream).await?;
    let mut reader = BytesReader::new(&request_blob);

    let header_hash = OpaqueHash::decode(&mut reader)?;
    let direction = u8::decode(&mut reader)?;
    let num_blocks = u32::decode(&mut reader)?;

    log::debug!("Block request received: hash={} direction={} num_blocks={}", hex::encode(&header_hash), direction, num_blocks);

    let mut response: Vec<u8> = Vec::new();
    let mut current_hash = header_hash;

    for _ in 0..num_blocks {
        match direction {
            // Ascending exclusive: skip the requested block, start from its child
            0 => {
                // TODO: requires a parent->child index, not yet implemented
                log::error!("Ascending exclusive block request not yet supported");
                break;
            }
            // Descending inclusive: start from the requested block, then parent, etc.
            1 => {
                if let Some(block) = get_block(&current_hash) {
                    current_hash = block.header.unsigned.parent;
                    block.encode_to(&mut response);
                } else {
                    log::debug!("Block {} not found in store", hex::encode(&current_hash));
                    break;
                }
            }
            _ => break,
        }
    }

    NetworkMessage::reply(response, &mut send_stream).await
}

async fn block_request(request_info: BlockRequestInfo, connection: Connection) -> Result<Vec<Block>, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    let payload = [request_info.header_hash.encode(), vec![request_info.direction], request_info.num_blocks.to_le_bytes().to_vec()].concat();
    NetworkMessage::send(BLOCK_REQUEST, payload, &mut send_stream).await?;
    log::debug!("Block request sent from block {} num_blocks: {:?}", hex::encode(&request_info.header_hash), request_info.num_blocks);

    let blocks_blob = NetworkMessage::recv(&mut recv_stream).await?;
    log::debug!("Blocks received: {:?} bytes", blocks_blob.len());
    
    let mut blocks = vec![];
    let mut reader = BytesReader::new(&blocks_blob);
    
    for i in 0..request_info.num_blocks {
        
        let block = Block::decode(&mut reader).map_err(|e| {
            log::error!("Failed to decode block {i}: {:?}", e);
            NetworkError::Decode(e)
        })?;

        //log::debug!("Slot: {:?} Block: {:x?}", block.header.unsigned.slot, block);
        blocks.push(block);
    }    

    Ok(blocks)
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
    log::debug!("Synced parent header: {}", hex::encode(&imported_blocks_stored.last_finalized_block.header_hash));
    
    // Calc state root
    let state_root = trie::merkle_state(&serialization::serialize(&last_global_finalized_state).map);
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
        log::debug!("SYNC process block {}", hex::encode(&sp_core::blake2_256(&block.header.encode())));
        store_block(block);
        match state_controller::stf(block) {
            Ok(_) => { log::debug!("processed successfully"); },
            Err(e) => { log::error!("error processing block: {:?}", e); },
        }
    }

    Ok(())
}

static IMPORTED_BLOCKS: LazyLock<Mutex<ImportedBlocks>> = LazyLock::new(|| { Mutex::new(ImportedBlocks::default()) });

fn is_synced(imported_blocks: &ImportedBlocks) -> bool {

    let mut imported_blocks_stored = IMPORTED_BLOCKS.lock().unwrap();

    if *imported_blocks_stored == *imported_blocks {
        return true;
    }

    *imported_blocks_stored = imported_blocks.clone();

    false
}

static LAST_ANNOUNCEMENT: LazyLock<Mutex<Announcement>> = LazyLock::new(|| { Mutex::new(Announcement::default()) });

pub fn set_last_announcement(announcement: Announcement) {
    *LAST_ANNOUNCEMENT.lock().unwrap() = announcement;
}

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

    true
}

pub async fn block_announcement(
    connection: Connection,
    send_stream: &mut SendStream,
    recv_stream: &mut RecvStream,
    handshake: Handshake,
    mut announcement_rx: mpsc::Receiver<Vec<u8>>,
) -> Result<(), NetworkError> {

    log::debug!("Last finalized block: {} slot: {:?}", hex::encode(&handshake.last_finalized_block.header_hash), handshake.last_finalized_block.slot);
    log::debug!("Leafs: {} Slots: {:?}"
    , handshake.leafs.iter().map(|leaf| hex::encode(&leaf.header_hash)).collect::<Vec<_>>().join(", "),  handshake.leafs.iter().map(|leaf| leaf.slot).collect::<Vec<TimeSlot>>());

    let imported_blocks = ImportedBlocks {
        last_finalized_block: handshake.last_finalized_block,
        leafs: handshake.leafs
    };

    if let Err(e) = sync_blocks(imported_blocks, connection.clone()).await {
        log::error!("Failed to sync blocks: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            result = NetworkMessage::recv(recv_stream) => {
                let announcement_blob = result?;

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
                log::debug!("Import block {} parent {}", hex::encode(&header_hash), hex::encode(&announcement.header.unsigned.parent));
                let request_info = BlockRequestInfo {
                    header_hash,
                    direction: 1,
                    num_blocks: 1
                };

                let blocks = match block_request(request_info, connection.clone()).await {
                    Ok(b) => b,
                    Err(e) => {
                        log::error!("Failed to request block: {:?}", e);
                        continue;
                    }
                };

                if let Some(block) = blocks.first() {
                    log::debug!("process block in loop {}", hex::encode(&sp_core::blake2_256(&block.header.encode())));
                    enqueue_block(block.clone()).await;
                } else {
                    log::error!("Block request returned empty response");
                }
            }

            Some(announcement_blob) = announcement_rx.recv() => {
                let len_bytes = (announcement_blob.len() as u32).to_le_bytes();
                if let Err(e) = send_stream.write_all(&len_bytes).await {
                    log::error!("Failed to send announcement length to {}: {:?}", connection.remote_address(), e);
                    continue;
                }
                if let Err(e) = send_stream.write_all(&announcement_blob).await {
                    log::error!("Failed to send announcement payload to {}: {:?}", connection.remote_address(), e);
                }
            }
        }
    }
}

