use codec::generic_codec::decode_from_bytes;
use constants::node::VALIDATORS_COUNT;
use crate::message::{BLOCK_ANNOUNCEMENT, BLOCK_REQUEST, TICKET_GENERATION, TICKET_PROXY};
use crate::{message, message::NetworkMessage, dev_accounts, node_config, grid};
use crate::jamnp_types::{NetworkError, Handshake, StreamKind};
use jam_types::*;
use quinn::{Connection, RecvStream, SendStream, Endpoint};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::select;
use tools::{hex, log};

#[derive(Debug, Clone, PartialEq)]
pub enum PeerState {
    Connected,
    Disconnected { retry_count: u32 },
    Reconnecting,
}

pub struct PeerInfo {
    pub handle: Option<PeerHandle>,
    pub state: PeerState,
    pub we_initiate: bool,
    pub is_neighbour: bool,
}

pub struct NetworkController {
    endpoint: Endpoint,
    peers: RwLock<HashMap<ValidatorIndex, PeerInfo>>,
}

impl NetworkController {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            peers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn init_peers(&self, my_index: ValidatorIndex, ed25519_keys: &[Ed25519Public]) {
        let my_key = &ed25519_keys[my_index as usize];
        let mut peers = self.peers.write().await;

        for index in 0..VALIDATORS_COUNT as ValidatorIndex {
            if index == my_index {
                continue;
            }
            let peer_key = &ed25519_keys[index as usize];
            peers.insert(index, PeerInfo {
                handle: None,
                state: PeerState::Disconnected { retry_count: 0 },
                we_initiate: grid::am_i_the_preferred_initiator(my_key, peer_key),
                is_neighbour: grid::is_neighbour(my_index, index),
            });
        }
    }

    pub async fn listen_network(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        while let Some(conn) = self.endpoint.accept().await {

            log::info!("Incoming connection attempt from {}", conn.remote_address());

            let this = self.clone();

            tokio::spawn(async move {
                match conn.await {
                    Ok(connection) => {

                        let id_account = connection.remote_address().port().saturating_sub(40000);
                        let identities = dev_accounts::parse_dev_accounts();

                        if id_account as usize >= identities.len() {
                            log::error!(
                                "Invalid peer index {} from port {}, ignoring connection",
                                id_account, connection.remote_address().port()
                            );
                            return;
                        }

                        let peer_index = id_account as ValidatorIndex;

                        log::info!(
                            "New connection established from {} bandersnatch public: {}",
                            connection.remote_address(),
                            hex::encode(&identities[id_account as usize].bandersnatch_public)
                        );

                        dev_accounts::add_dev_account(
                            identities[id_account as usize].bandersnatch_public,
                            connection.clone(),
                        );

                        let (tx, rx) = mpsc::channel::<PeerCommand>(32);
                        let handle = PeerHandle {
                            connection: connection.clone(),
                            sender: tx,
                            announcement_tx: Arc::new(std::sync::Mutex::new(None)),
                        };

                        {
                            let mut peers = this.peers.write().await;
                            if let Some(info) = peers.get_mut(&peer_index) {
                                info.handle = Some(handle.clone());
                                info.state = PeerState::Connected;
                            } else {
                                peers.insert(peer_index, PeerInfo {
                                    handle: Some(handle.clone()),
                                    state: PeerState::Connected,
                                    we_initiate: false,
                                    is_neighbour: grid::is_neighbour(node_config::get_account_id(), peer_index),
                                });
                            }
                        }

                        handle.peer_task(rx).await;

                        {
                            let mut peers = this.peers.write().await;
                            if let Some(info) = peers.get_mut(&peer_index) {
                                info.handle = None;
                                info.state = PeerState::Disconnected { retry_count: 0 };
                            }
                        }

                        dev_accounts::remove_dev_account(
                            &identities[id_account as usize].bandersnatch_public,
                        );

                        log::info!("Server connection task finished for {}", id_account);
                    }
                    Err(e) => {
                        log::error!("Connection error: {}", e);
                    }
                }
            });
        }

        self.endpoint.wait_idle().await;
        Ok(())
    }

    pub async fn broadcast_announcement(&self, announcement_blob: Vec<u8>) {
        let targets: Vec<(ValidatorIndex, mpsc::Sender<Vec<u8>>)> = {
            let peers = self.peers.read().await;
            peers.iter()
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
            if let Err(e) = tx.send(announcement_blob.clone()).await {
                log::error!("Failed to send announcement to peer {}: {:?}", peer_index, e);
            }
            log::info!("Broadcast announcement to peer {peer_index}");
        }
    }

    pub async fn connect_to_peer(
        self: Arc<Self>,
        peer_index: ValidatorIndex,
    ) -> Result<(), NetworkError> {

        {
            let peers = self.peers.read().await;
            if let Some(info) = peers.get(&peer_index) {
                if info.state == PeerState::Connected {
                    return Ok(());
                }
            }
        }

        let identities = dev_accounts::parse_dev_accounts();
        let node_alt_name = identities[peer_index as usize].dns_alt_name.clone();
        let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000 + peer_index);

        log::info!(
            "Attempt connection to {} bandersnatch public: {}",
            node_addr,
            hex::encode(&identities[peer_index as usize].bandersnatch_public)
        );

        let connecting = self.endpoint.connect(node_addr, &node_alt_name)?;
        let connection = connecting.await?;

        log::info!("Connected to {}", node_addr);

        dev_accounts::add_dev_account(
            identities[peer_index as usize].bandersnatch_public,
            connection.clone(),
        );

        let (tx, rx) = mpsc::channel(32);
        let handle = PeerHandle {
            connection: connection.clone(),
            sender: tx,
            announcement_tx: Arc::new(std::sync::Mutex::new(None)),
        };

        {
            let mut peers = self.peers.write().await;
            if let Some(info) = peers.get_mut(&peer_index) {
                info.handle = Some(handle.clone());
                info.state = PeerState::Connected;
            } else {
                peers.insert(peer_index, PeerInfo {
                    handle: Some(handle.clone()),
                    state: PeerState::Connected,
                    we_initiate: true,
                    is_neighbour: grid::is_neighbour(node_config::get_account_id(), peer_index),
                });
            }
        }

        let peer_handle = handle.clone();

        tokio::spawn(async move {
            if let Err(e) = handle.open_stream(BLOCK_ANNOUNCEMENT).await {
                log::error!("Failed to open announcement stream: {:?}", e);
            }
        });

        peer_handle.peer_task(rx).await;

        {
            let mut peers = self.peers.write().await;
            if let Some(info) = peers.get_mut(&peer_index) {
                info.handle = None;
                info.state = PeerState::Disconnected { retry_count: 0 };
            }
        }

        dev_accounts::remove_dev_account(
            &identities[peer_index as usize].bandersnatch_public,
        );

        log::info!("Peer connection task finished for {}", node_addr);

        Ok(())
    }

    pub async fn connection_monitor(self: Arc<Self>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            let reconnect_targets: Vec<(ValidatorIndex, u32, bool)> = {
                let peers = self.peers.read().await;
                peers.iter()
                    .filter_map(|(&index, info)| {
                        if let PeerState::Disconnected { retry_count } = info.state {
                            Some((index, retry_count, info.we_initiate))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Log network status summary
            {
                let peers = self.peers.read().await;
                let total = peers.len();
                let connected = peers.values().filter(|i| i.state == PeerState::Connected).count();
                let neighbours_connected = peers.values()
                    .filter(|i| i.state == PeerState::Connected && i.is_neighbour)
                    .count();
                let neighbours_total = peers.values().filter(|i| i.is_neighbour).count();
                log::info!(
                    "Network status: {}/{} peers connected, neighbours {}/{}",
                    connected, total, neighbours_connected, neighbours_total
                );
            }

            for (peer_index, retry_count, we_initiate) in reconnect_targets {
                let backoff = if we_initiate {
                    calculate_backoff(retry_count)
                } else {
                    std::time::Duration::from_secs(5)
                };
                log::info!(
                    "Reconnecting to peer {} (retry #{}, backoff {}s)",
                    peer_index, retry_count, backoff.as_secs()
                );

                // Mark as Reconnecting and increment retry_count for next round
                {
                    let mut peers = self.peers.write().await;
                    if let Some(info) = peers.get_mut(&peer_index) {
                        info.state = PeerState::Reconnecting;
                    }
                }

                let this = self.clone();
                let next_retry = retry_count + 1;

                tokio::spawn(async move {
                    tokio::time::sleep(backoff).await;
                    if let Err(e) = this.clone().connect_to_peer(peer_index).await {
                        log::error!("Reconnection to peer {} failed: {:?}", peer_index, e);
                        // connect_to_peer sets Disconnected(retry=0) on disconnect,
                        // but if it fails before connecting, we need to set the incremented retry_count
                        let mut peers = this.peers.write().await;
                        if let Some(info) = peers.get_mut(&peer_index) {
                            if info.state != PeerState::Connected {
                                info.state = PeerState::Disconnected { retry_count: next_retry };
                            }
                        }
                    }
                });
            }
        }
    }
}

pub fn calculate_backoff(retry_count: u32) -> std::time::Duration {
    let secs = match retry_count {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        5 => 32,
        _ => 60,
    };
    std::time::Duration::from_secs(secs)
}

#[derive(Clone, Debug)]
pub enum PeerCommand {
    BlockAnnouncement,
    CloseConnection,
}

#[derive(Clone, Debug)]
pub struct PeerHandle {
    pub connection: Connection,
    pub sender: mpsc::Sender<PeerCommand>,
    pub announcement_tx: Arc<std::sync::Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
}

impl PeerHandle {

    pub async fn open_stream(&self, kind: StreamKind) -> Result<(), NetworkError> {

        let (mut send_stream, mut recv_stream) = self.connection.open_bi().await?;
        
        match kind {
            BLOCK_ANNOUNCEMENT => {
                log::info!("Open stream {BLOCK_ANNOUNCEMENT} for address: {:?}", self.connection.remote_address());
                let handshake: Vec<u8> = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                NetworkMessage::send_up(BLOCK_ANNOUNCEMENT, handshake, &mut send_stream).await?;
                let handshake = decode_from_bytes::<Handshake>(&NetworkMessage::recv(&mut recv_stream).await?)?;
                let connection = self.connection.clone();
                let (ann_tx, ann_rx) = mpsc::channel::<Vec<u8>>(16);
                *self.announcement_tx.lock().unwrap() = Some(ann_tx);
                let announcement_tx_ref = self.announcement_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = message::block::announcement(connection, &mut send_stream, &mut recv_stream, handshake, ann_rx).await {
                        log::error!("Block announcement stream ended: {:?}", e);
                    }
                    *announcement_tx_ref.lock().unwrap() = None;
                });
            },
            _ => {
                log::error!("Unknown stream kind: {:?}", kind);
            }
        }

        Ok(())
    }

    async fn handle_stream(&self, mut send_stream: SendStream, mut recv_stream: RecvStream) -> Result<(), NetworkError> {

        let mut stream_kind_buf = [0u8; 1];
        recv_stream.read_exact(&mut stream_kind_buf).await?;

        let kind = u8::from_le_bytes(stream_kind_buf);

        match kind {

            BLOCK_ANNOUNCEMENT => {
                let connection = self.connection.clone();
                let handshake: Vec<u8> = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                let len_bytes = (handshake.len() as u32).to_le_bytes();
                send_stream.write_all(&([len_bytes.to_vec(), handshake].concat())).await.ok();
                let handshake = decode_from_bytes::<Handshake>(&NetworkMessage::recv(&mut recv_stream).await?)?;
                let (ann_tx, ann_rx) = mpsc::channel::<Vec<u8>>(16);
                *self.announcement_tx.lock().unwrap() = Some(ann_tx);
                let announcement_tx_ref = self.announcement_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = message::block::announcement(connection, &mut send_stream, &mut recv_stream, handshake, ann_rx).await {
                        log::error!("Block announcement stream ended: {:?}", e);
                    }
                    *announcement_tx_ref.lock().unwrap() = None;
                });
            },
            BLOCK_REQUEST => {
                log::debug!("Block request received from {}", self.connection.remote_address());
                tokio::spawn(async move {
                    if let Err(e) = message::block::handle_request(send_stream, recv_stream).await {
                        log::error!("Failed to handle block request: {:?}", e);
                    }
                });
            },
            TICKET_GENERATION => {
                log::debug!("Generated ticket received -> Send to all current validators");
                tokio::spawn(async move {
                    if let Err(e) = message::ticket::recv_from_generator(recv_stream).await {
                        log::error!("Failed to handle ticket from generator: {:?}", e);
                    }
                });
            },
            TICKET_PROXY => {
                log::debug!("Received ticket from proxy -> Include in a block");
                tokio::spawn(async move {
                    if let Err(e) = message::ticket::recv_distribution(recv_stream).await {
                        log::error!("Failed to handle ticket distribution: {:?}", e);
                    }
                });
            },
            _ => {
                log::error!("Unknown stream kind: {:?}", kind);
            },
        }

        Ok(())
    }

    pub async fn peer_task(&self, mut cmd_rx: mpsc::Receiver<PeerCommand>) {

        log::info!("New connection established from {}", self.connection.remote_address());

        loop {
            select! {
                incoming = self.connection.accept_bi() => {
                    match incoming {
                        Ok((send_stream, recv_stream)) => {
                            if let Err(e) = self.handle_stream(send_stream, recv_stream).await {
                                log::error!("Failed to handle stream from {}: {:?}", self.connection.remote_address(), e);
                            }
                        }
                        Err(e) => {
                            log::info!("Connection closed from {}: {:?}", self.connection.remote_address(), e);
                            break;
                        }
                    }
                }

                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(PeerCommand::BlockAnnouncement) => {
                            log::info!("Sending block announcement to {}", self.connection.remote_address());

                        }
                        Some(PeerCommand::CloseConnection) => {
                            log::info!("Closing connection to {}", self.connection.remote_address());
                            let _ = self.connection.close(0u32.into(), b"close");
                            break;
                        }
                        None => {
                            log::info!("Command channel closed for {}", self.connection.remote_address());
                            break;
                        }
                    }
                }
            }
        }

        //self.connection.close(0, b"Task finalized");

        log::info!("peer_task: connection loop finished for {}", self.connection.remote_address());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_backoff_test() {
        assert_eq!(calculate_backoff(0), std::time::Duration::from_secs(1));
        assert_eq!(calculate_backoff(1), std::time::Duration::from_secs(2));
        assert_eq!(calculate_backoff(2), std::time::Duration::from_secs(4));
        assert_eq!(calculate_backoff(3), std::time::Duration::from_secs(8));
        assert_eq!(calculate_backoff(4), std::time::Duration::from_secs(16));
        assert_eq!(calculate_backoff(5), std::time::Duration::from_secs(32));
        assert_eq!(calculate_backoff(6), std::time::Duration::from_secs(60));
        assert_eq!(calculate_backoff(100), std::time::Duration::from_secs(60));
    }
}
