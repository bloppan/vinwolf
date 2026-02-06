use codec::generic_codec::decode_from_bytes;
use crate::message::{BLOCK_ANNOUNCEMENT, BLOCK_REQUEST, TICKET_GENERATION, TICKET_PROXY};
use crate::{message, message::NetworkMessage, dev_accounts, net_utils, node_config};
use crate::jamnp_types::{ConnectionError, NetworkError, Handshake, StreamKind};
use jam_types::ValidatorIndex;
use quinn::{Connection, RecvStream, SendStream, Endpoint};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::select;
use tools::{hex, log};

pub struct NetworkController {
    endpoint: Endpoint,
    peers: RwLock<HashMap<ValidatorIndex, PeerHandle>>,
}

impl NetworkController {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            peers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn listen_network(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        while let Some(conn) = self.endpoint.accept().await {
            
            log::info!("Incoming connection attempt from {}", conn.remote_address());

            /*if conn.remote_address().port() != 40004 {
                continue;
            }*/

            let this = self.clone();

            tokio::spawn(async move {
                match conn.await {
                    Ok(connection) => {
                        
                        let id_account = connection.remote_address().port().saturating_sub(40000);
                        let dev_accounts = dev_accounts::parse_dev_accounts();

                        log::info!(
                            "New connection established from {} bandersnatch public: {}",
                            connection.remote_address(),
                            hex::encode(&dev_accounts[id_account as usize].bandersnatch_public)
                        );

                        dev_accounts::add_dev_account(
                            dev_accounts[id_account as usize].bandersnatch_public,
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
                            peers.insert(id_account as ValidatorIndex, handle.clone());
                        }

                        handle.peer_task(rx).await;

                        {
                            let mut peers = this.peers.write().await;
                            peers.remove(&(id_account as ValidatorIndex));
                        }

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
        let my_index = node_config::get_account_id();
        let peers = self.peers.read().await;
        for (&peer_index, handle) in peers.iter() {
            if !net_utils::is_grid_neighbour(my_index, peer_index) {
                continue;
            }
            let tx = handle.announcement_tx.lock().unwrap().clone();
            if let Some(tx) = tx {
                if let Err(e) = tx.send(announcement_blob.clone()).await {
                    log::error!("Failed to send announcement to peer {}: {:?}", peer_index, e);
                }
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
            if let Some(_handle) = peers.get(&peer_index) {
                return Ok(());
            }
        }

        let dev_accounts = dev_accounts::parse_dev_accounts();
        let node_alt_name = dev_accounts[peer_index as usize].dns_alt_name.clone();
        let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000 + peer_index);

        log::info!(
            "Attempt connection to {} bandersnatch public: {}",
            node_addr,
            hex::encode(&dev_accounts[peer_index as usize].bandersnatch_public)
        );

        let connecting = self.endpoint.connect(node_addr, &node_alt_name).unwrap();
        let connection = connecting.await.unwrap();

        log::info!("Connected to {}", node_addr);

        let (tx, rx) = mpsc::channel(32);
        let handle = PeerHandle {
            connection: connection.clone(),
            sender: tx,
            announcement_tx: Arc::new(std::sync::Mutex::new(None)),
        };

        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_index, handle.clone());
        }

        let peer_handle = handle.clone();

        tokio::spawn( async move {
            handle.open_stream(BLOCK_ANNOUNCEMENT).await.unwrap();
        });

        peer_handle.peer_task(rx).await;
        
        {
            let mut peers = self.peers.write().await;
            peers.remove(&(peer_index as ValidatorIndex));
        }

        log::info!("Peer connection task finished for {}", node_addr);

        Ok(())
    }

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

        let (mut send_stream, mut recv_stream) = self.connection.open_bi().await.map_err(|e| {
            log::error!("Failed to open bidirectional stream: {:?}", e);
            NetworkError::ConnectionError(ConnectionError::OpenBidirectionalStream)
        })?;

        match kind {
            BLOCK_ANNOUNCEMENT => {
                log::info!("Open stream {BLOCK_ANNOUNCEMENT} for address: {:?}", self.connection.remote_address());
                let handshake: Vec<u8> = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                NetworkMessage::send_up(BLOCK_ANNOUNCEMENT, handshake, &mut send_stream).await.unwrap();
                let handshake = decode_from_bytes::<Handshake>(&NetworkMessage::recv(&mut recv_stream).await?).unwrap();
                let connection = self.connection.clone();
                let (ann_tx, ann_rx) = mpsc::channel::<Vec<u8>>(16);
                *self.announcement_tx.lock().unwrap() = Some(ann_tx);
                tokio::spawn(async move {
                    message::block_announcement(connection, &mut send_stream, &mut recv_stream, handshake, ann_rx).await.unwrap();
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
        recv_stream.read_exact(&mut stream_kind_buf).await.unwrap();

        let kind = u8::from_le_bytes(stream_kind_buf);
        
        match kind {

            BLOCK_ANNOUNCEMENT => {
                let connection = self.connection.clone();
                let handshake: Vec<u8> = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                let len_bytes = (handshake.len() as u32).to_le_bytes();
                send_stream.write_all(&([len_bytes.to_vec(), handshake].concat())).await.ok();
                let handshake = decode_from_bytes::<Handshake>(&NetworkMessage::recv(&mut recv_stream).await?).unwrap();
                let (ann_tx, ann_rx) = mpsc::channel::<Vec<u8>>(16);
                *self.announcement_tx.lock().unwrap() = Some(ann_tx);
                tokio::spawn(async move {
                    message::block_announcement(connection, &mut send_stream, &mut recv_stream, handshake, ann_rx).await.unwrap();
                });
            },
            BLOCK_REQUEST => {
                log::debug!("Block request received from {}", self.connection.remote_address());
                tokio::spawn(async move {
                    if let Err(e) = message::handle_block_request(send_stream, recv_stream).await {
                        log::error!("Failed to handle block request: {:?}", e);
                    }
                });
            },
            TICKET_GENERATION => {
                log::debug!("Generated ticket received -> Send to all current validators");
                tokio::spawn(async move {
                    message::recv_ticket_from_generator(recv_stream).await.unwrap();
                });
            },
            TICKET_PROXY => {
                log::debug!("Received ticket from proxy -> Include in a block");
                tokio::spawn(async move {
                    message::recv_ticket_distribution(recv_stream).await.unwrap();
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
                            self.handle_stream(send_stream, recv_stream).await.unwrap();
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