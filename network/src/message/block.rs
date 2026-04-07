use crate::message::*;
use jam_types::*;
use std::sync::Arc;

pub struct BlockRequestInfo {
    pub header_hash: OpaqueHash,
    pub direction: u8,
    pub num_blocks: u32,
}

pub enum Direction {
    AscendingExclusive = 0,
    DescendingInclusive = 1,
}

pub async fn handle_request(mut send_stream: SendStream, mut recv_stream: RecvStream) -> Result<(), NetworkError> {

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
                if let Some(block) = block::get(&current_hash) {
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

    NetworkMessage::ce_reply(response, &mut send_stream).await
}

pub async fn request(request_info: BlockRequestInfo, connection: Connection) -> Result<Vec<Block>, NetworkError> {

    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    let payload = [request_info.header_hash.encode(), vec![request_info.direction], request_info.num_blocks.to_le_bytes().to_vec()].concat();
    NetworkMessage::ce_send(BLOCK_REQUEST, payload, &mut send_stream).await?;
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

pub mod announcement {
    use super::*;
    use crate::net_ctrl::{NetworkController, PeerState};

    fn is_new(announcement: &Announcement) -> bool {
        let slot: TimeSlot = announcement.header.unsigned.slot;
        let prev: u32 = LAST_SEEN_SLOT.fetch_max(slot, Ordering::AcqRel);
        slot > prev
    }

    pub fn build(header: Header) -> Vec<u8> {
        let parent_hash: OpaqueHash = ::block::header::get_parent_header();
        let time: TimeSlot = state_handler::time::get();

        let announcement = Announcement {
            header,
            last_finalized_block: LastFinalizedBlock {
                header_hash: parent_hash,
                slot: time,
            },
        };

        announcement.encode()
    }

    pub async fn run(
        connection: Connection,
        send_stream: &mut SendStream,
        recv_stream: &mut RecvStream,
        handshake: Handshake,
        mut announcement_rx: mpsc::Receiver<Arc<[u8]>>,
    ) -> Result<(), NetworkError> {
        log::debug!(
            "Last finalized block: {} slot: {:?}",
            hex::encode(&handshake.last_finalized_block.header_hash),
            handshake.last_finalized_block.slot
        );
        log::debug!(
            "Leafs: {} Slots: {:?}",
            handshake
                .leafs
                .iter()
                .map(|leaf| hex::encode(&leaf.header_hash))
                .collect::<Vec<_>>()
                .join(", "),
            handshake.leafs.iter().map(|leaf| leaf.slot).collect::<Vec<TimeSlot>>()
        );

        let imported_blocks = ImportedBlocks {
            last_finalized_block: handshake.last_finalized_block,
            leafs: handshake.leafs,
        };

        if let Err(e) = sync_blocks(imported_blocks, connection.clone()).await {
            log::error!("Failed to sync blocks: {:?}", e);
            return Err(e);
        }

        loop {
            tokio::select! {
                // Receive announcement from peers
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

                    let blocks = match block::request(request_info, connection.clone()).await {
                        Ok(b) => b,
                        Err(e) => {
                            log::error!("Failed to request block: {:?}", e);
                            continue;
                        }
                    };

                    if let Some(block) = blocks.first() {
                        log::debug!("process block in loop {}", hex::encode(&sp_core::blake2_256(&block.header.encode())));
                        enqueue(block.clone()).await;
                    } else {
                        log::error!("Block request returned empty response");
                    }
                }

                // Send announcement to peers 
                Some(announcement_blob) = announcement_rx.recv() => {
                    if let Err(e) = NetworkMessage::up_send(announcement_blob.as_ref(), send_stream).await {
                        log::error!("Failed to send announcement to {}: {:?}", connection.remote_address(), e);
                    }
                }
            }
        }
    }

    pub async fn broadcast(network: &NetworkController, announcement_blob: Arc<[u8]>) {
        let targets: Vec<(ValidatorIndex, mpsc::Sender<Arc<[u8]>>)> = {
            let peers = network.peers.read().await;
            peers
                .iter()
                .filter(|(_, info)| info.state == PeerState::Connected && info.is_neighbour)
                .filter_map(|(&peer_index, info)| {
                    info.handle.as_ref().and_then(|handle| {
                        let tx = handle.announcement_tx.lock().unwrap().clone();
                        tx.map(|tx| (peer_index, tx))
                    })
                })
                .collect()
        };

        for (peer_index, tx) in targets {
            match tx.try_send(Arc::clone(&announcement_blob)) {
                Ok(()) => {
                    log::info!("Broadcast announcement to peer {peer_index}");
                }
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    log::warn!("Announcement channel full for peer {}", peer_index);
                }
                Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                    log::warn!("Announcement channel closed for peer {}", peer_index);
                }
            }
        }
    }
}

static BLOCK_STORE: LazyLock<Mutex<HashMap<OpaqueHash, Block>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn store(block: &Block) {
    let header_hash = sp_core::blake2_256(&block.header.encode());
    log::debug!("Storing block {}", hex::encode(&header_hash));
    BLOCK_STORE.lock().unwrap().insert(header_hash, block.clone());
}

pub fn get(header_hash: &OpaqueHash) -> Option<Block> {
    BLOCK_STORE.lock().unwrap().get(header_hash).cloned()
}

// --- Block processing queue ---

pub struct QueuedBlock {
    pub block: Block,
    pub result_tx: Option<oneshot::Sender<Result<(), ImportError>>>,
}

static BLOCK_QUEUE_TX: OnceLock<mpsc::Sender<QueuedBlock>> = OnceLock::new();

pub fn init_queue(buffer: usize) -> mpsc::Receiver<QueuedBlock> {
    let (tx, rx) = mpsc::channel(buffer);
    BLOCK_QUEUE_TX.set(tx).expect("init_block_queue called more than once");
    rx
}

pub async fn enqueue(block: Block) {
    let tx = BLOCK_QUEUE_TX.get().expect("block queue not initialized");
    let queued = QueuedBlock { block, result_tx: None };
    if let Err(e) = tx.send(queued).await {
        log::error!("Failed to enqueue block: {:?}", e);
    }
}

pub async fn enqueue_and_wait(block: Block) -> Result<(), ImportError> {
    let tx = BLOCK_QUEUE_TX.get().expect("block queue not initialized");
    let (result_tx, result_rx) = oneshot::channel();
    let queued = QueuedBlock { block, result_tx: Some(result_tx) };
    tx.send(queued).await.map_err(|_| {
        log::error!("Failed to enqueue block for processing");
        ImportError::HeaderError(HeaderErrorCode::BadParentHeader)
    })?;
    result_rx.await.unwrap_or_else(|_| {
        log::error!("Block queue consumer dropped without sending result");
        Err(ImportError::HeaderError(HeaderErrorCode::BadParentHeader))
    })
}

pub async fn run_queue(mut rx: mpsc::Receiver<QueuedBlock>) {
    
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
                store(&qb.block);
                let result = state_ctrl::stf(&qb.block);
                match &result {
                    Ok(_) => {
                        update_last_imported_slot(slot);
                        log::debug!("Queue: block slot={} processed successfully", slot);
                    }
                    Err(e) => log::error!("Queue: error processing block slot={}: {:?}", slot, e),
                }
                if let Some(tx) = qb.result_tx {
                    let _ = tx.send(result);
                }
            }
        }
    }
}

async fn sync_blocks(imported_blocks_recv: ImportedBlocks, connection: Connection) -> Result<(), NetworkError> {

    // Quick check without lock — avoids contention
    if is_synced() {
        log::debug!("Already synced (quick check), skipping sync from {:?}", connection.remote_address());
        return Ok(());
    }

    // Acquire lock — only one thread syncs at a time, others wait here
    let _guard = SYNC_LOCK.lock().await;

    // Re-check after acquiring lock — another thread may have synced already
    if is_synced() {
        log::debug!("Already synced (after lock), skipping sync from {:?}", connection.remote_address());
        return Ok(());
    }

    log::debug!("Syncing blocks from address: {:?}", connection.remote_address());

    let last_global_finalized_state = state::request(imported_blocks_recv.last_finalized_block.header_hash, connection.clone()).await?;
    ::block::header::set_parent_header(imported_blocks_recv.last_finalized_block.header_hash);
    log::debug!("Synced parent header: {}", hex::encode(&imported_blocks_recv.last_finalized_block.header_hash));

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

    let leaf = match imported_blocks_recv.leafs.first() {
        Some(l) => l,
        None => {
            log::error!("sync_blocks: no leafs in imported blocks");
            return Ok(());
        }
    };

    let request_info = BlockRequestInfo {
                            header_hash: leaf.header_hash,
                            direction: 1,
                            num_blocks: leaf.slot - imported_blocks_recv.last_finalized_block.slot
    };

    log::debug!("Request blocks from slot {:?} in order to sync", Some(leaf.slot));
    let mut blocks = block::request(request_info, connection.clone()).await?;
    blocks.sort_by_key(|block| block.header.unsigned.slot);

    for block in blocks.iter() {
        log::debug!("SYNC process block {}", hex::encode(&sp_core::blake2_256(&block.header.encode())));
        block::store(block);
        match state_ctrl::stf(block) {
            Ok(_) => {
                update_last_imported_slot(block.header.unsigned.slot);
                log::debug!("processed successfully");
            },
            Err(e) => { log::error!("error processing block: {:?}", e); },
        }
    }

    Ok(())
}

// Slot of the last successfully imported block
static LAST_IMPORTED_SLOT: AtomicU32 = AtomicU32::new(0);

// Lock to serialize the entire sync operation
static SYNC_LOCK: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));

const MAX_SLOTS_BEHIND: u32 = 10;

fn is_synced() -> bool {
    let last_slot = LAST_IMPORTED_SLOT.load(Ordering::Acquire);
    match time::current_slot() {
        Ok(current) => current.saturating_sub(last_slot) <= MAX_SLOTS_BEHIND,
        Err(_) => false,
    }
}

pub fn update_last_imported_slot(slot: TimeSlot) {
    LAST_IMPORTED_SLOT.fetch_max(slot, Ordering::Release);
}

static LAST_SEEN_SLOT: AtomicU32 = AtomicU32::new(0);

pub fn mark_slot_seen(slot: TimeSlot) {
    LAST_SEEN_SLOT.fetch_max(slot, Ordering::Release);
}
