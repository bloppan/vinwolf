use crate::{dev_accounts, grid, message, net_utils, node_config, NetworkController};
use jam_types::ValidatorIndex;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tools::log;

pub type InitResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub struct NetworkRuntime {
    pub controller: Arc<NetworkController>,
    pub server_handle: JoinHandle<()>,
}

pub async fn init_network(validator_index: ValidatorIndex) -> InitResult<NetworkRuntime> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;

    let (certs, key_der) = net_utils::load_credentials(validator_index)?;

    let server_config = net_utils::load_server_config(certs.clone(), key_der.clone_key())?;
    let client_config = net_utils::load_client_config(certs, key_der)?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000 + validator_index);
    log::debug!("Listening on {}", bind_addr);
    let mut endpoint = quinn::Endpoint::server(server_config, bind_addr)?;
    endpoint.set_default_client_config(client_config);

    let net = Arc::new(NetworkController::new(endpoint));

    let block_queue_rx = message::block::init_queue(32);
    tokio::spawn(async move {
        message::block::run_queue(block_queue_rx).await;
    });

    node_config::set_account_id(validator_index);

    let identities = dev_accounts::parse_dev_accounts();
    let ed25519_public: Vec<_> = identities.iter().map(|key| key.ed25519_public).collect();
    let own_ed25519_public = ed25519_public
        .get(validator_index as usize)
        .ok_or_else(|| format!("Validator index {} is out of range", validator_index))?;

    net.init_peers(validator_index, &ed25519_public).await;

    let server_handle = {
        let net_for_server = net.clone();
        tokio::spawn(async move {
            if let Err(e) = net_for_server.listen_network().await {
                log::error!("Server task failed: {:?}", e);
            }
        })
    };

    // Initial connection attempts (fire-and-forget, monitor will reconnect if they fail).
    for (index, ed25519_key) in ed25519_public.iter().enumerate() {
        if index == validator_index as usize {
            continue;
        }

        if grid::am_i_the_preferred_initiator(own_ed25519_public, ed25519_key) {
            log::info!("Initialize connection to node {:?}", index);
            let net_for_client = net.clone();
            tokio::spawn(async move {
                if let Err(e) = net_for_client
                    .connect_to_peer(index as ValidatorIndex)
                    .await
                {
                    log::error!("Client {} failed: {:?}", index, e);
                }
            });
        }
    }

    let net_for_monitor = net.clone();
    tokio::spawn(async move {
        net_for_monitor.connection_monitor().await;
    });

    Ok(NetworkRuntime {
        controller: net,
        server_handle,
    })
}
