use crate::message::BLOCK_ANNOUNCEMENT;
use crate::{message, message::NetworkMessage, message::ConnectionInfo, dev_accounts};
use crate::jamnp_types::{ConnectionError, NetworkError, StreamError, StreamKind};
use jam_types::{ValidatorIndex, Ed25519Public};
use quinn::{Connection, RecvStream, SendStream, Endpoint};
use utils::log;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv6Addr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::select;
use tokio::io::AsyncWriteExt;

pub fn am_i_the_preferred_initiator(my_key: &Ed25519Public, peer_key: &Ed25519Public) -> bool {
    let cond = ((my_key[31] > 127) ^ (peer_key[31] > 127)) ^ (my_key < peer_key);
    cond
}

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

    pub async fn run_server(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        while let Some(conn) = self.endpoint.accept().await {
            log::info!("Incoming connection attempt from {}", conn.remote_address());
            let this = self.clone();

            tokio::spawn(async move {
                match conn.await {
                    Ok(connection) => {
                        
                        let id_account = connection.remote_address().port().saturating_sub(40000);
                        let dev_accounts = dev_accounts::parse_dev_accounts();

                        log::info!(
                            "New connection established from {} bandersnatch public: {}",
                            connection.remote_address(),
                            utils::hex::encode(&dev_accounts[id_account as usize].bandersnatch_public)
                        );

                        dev_accounts::add_dev_account(
                            dev_accounts[id_account as usize].bandersnatch_public,
                            connection.clone(),
                        );

                        let (tx, rx) = mpsc::channel::<PeerCommand>(32);
                        let handle = PeerHandle {
                            connection: connection.clone(),
                            sender: tx,
                        };

                        {
                            let mut peers = this.peers.write().await;
                            peers.insert(id_account as ValidatorIndex, handle);
                        }

                        peer_task(connection, rx).await;

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

    pub async fn ensure_connected(
        self: &Arc<Self>,
        peer_index: ValidatorIndex,
        peer_pubkey: &Ed25519Public,
        my_pubkey: &Ed25519Public,
    ) -> Result<Option<PeerHandle>, NetworkError> {

        {
            let peers = self.peers.read().await;
            if let Some(handle) = peers.get(&peer_index) {
                return Ok(Some(handle.clone()));
            }
        }

        if !am_i_the_preferred_initiator(my_pubkey, peer_pubkey) {
            return Ok(None);
        }

        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::LOCALHOST), 
            40000 + peer_index
        );

        let connecting = self.endpoint.connect(addr, "localhost").unwrap();
        let connection = connecting.await.unwrap();

        let (tx, rx) = mpsc::channel(32);
        let handle = PeerHandle {
            connection: connection.clone(),
            sender: tx,
        };

        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_index, handle.clone());
        }

        tokio::spawn(peer_task(connection, rx));

        Ok(Some(handle))
    }

}

#[derive(Clone, Debug)]
pub enum PeerCommand {
    CloseConnection,
}

#[derive(Clone, Debug)]
pub struct PeerHandle {
    pub connection: Connection,
    pub sender: mpsc::Sender<PeerCommand>,
}

impl PeerHandle {
    pub async fn open_stream(&self, kind: StreamKind) -> Result<(), NetworkError> {

        let (send_stream, recv_stream) = self.connection.open_bi().await.map_err(|e| {
            log::error!("Failed to open bidirectional stream: {:?}", e);
            NetworkError::ConnectionError(ConnectionError::OpenBidirectionalStream)
        })?;

        let connection_info = ConnectionInfo {
            connection: self.connection.clone(),
            recv_stream,
            send_stream,
            kind
        };

        match kind {
            BLOCK_ANNOUNCEMENT => {
                log::info!("Open stream {BLOCK_ANNOUNCEMENT} for address: {:?}", self.connection.remote_address());
                
                let handshake: Vec<u8> = vec![15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0, 1, 15, 140, 101, 194, 104, 174, 233, 240, 82, 49, 141, 19, 229, 55, 117, 252, 165, 108, 150, 250, 80, 25, 40, 178, 168, 52, 196, 232, 108, 37, 140, 85, 138, 102, 59, 0];
                let (mut send_stream, _recv_stream) = self.connection.open_bi().await.map_err(|e| {
                    log::error!("Failed to open bidirectional stream: {:?}", e);
                    NetworkError::ConnectionError(ConnectionError::OpenBidirectionalStream)
                })?;
                NetworkMessage::send_up(BLOCK_ANNOUNCEMENT, handshake, &mut send_stream).await.unwrap();
                
                message::block_announcement(connection_info).await.unwrap();
            },
            _ => {
                log::error!("Unknown stream kind: {:?}", kind);
            }
        }
        Ok(())
    }
}


pub async fn peer_task(connection: Connection, mut cmd_rx: mpsc::Receiver<PeerCommand>) {
    log::info!("New connection established from {}", connection.remote_address());

    loop {
        select! {
            incoming = connection.accept_bi() => {
                match incoming {
                    Ok((send_stream, mut recv_stream)) => {
                        let conn_clone = connection.clone();
                        tokio::spawn(async move {
                            let mut stream_kind_buf = [0u8; 1];
                            if recv_stream.read_exact(&mut stream_kind_buf).await.is_ok() {
                                log::info!(
                                    "Received stream kind {:?} from peer: {:?}",
                                    stream_kind_buf,
                                    conn_clone.remote_address()
                                );
                                let conn_info = message::ConnectionInfo {
                                    connection: conn_clone,
                                    send_stream,
                                    recv_stream,
                                    kind: stream_kind_buf[0],
                                };
                                message::handle_stream(conn_info).await;
                            }
                        });
                        log::info!("Waiting for another stream");
                    }
                    Err(e) => {
                        log::info!("Connection closed from {}: {:?}", connection.remote_address(), e);
                        break;
                    }
                }
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(PeerCommand::CloseConnection) => {
                        log::info!("Closing connection to {}", connection.remote_address());
                        let _ = connection.close(0u32.into(), b"close");
                        break;
                    }
                    None => {
                        log::info!("Command channel closed for {}", connection.remote_address());
                        break;
                    }
                }
            }
        }
    }

    log::info!("peer_task: connection loop finished for {}", connection.remote_address());
}