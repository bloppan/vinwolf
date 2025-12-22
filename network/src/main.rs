pub mod dev_accounts;
pub mod message;
pub mod net_controller;
pub mod net_utils;
pub mod jamnp_codec;
pub mod jamnp_types;

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use jam_types::ValidatorIndex;
use network::node_config;
use utils::log;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn print_help() {    
    println!("vinwolf network");
    println!();
    println!("\x1b[1mUsage example:\x1b[0m\n");
    println!("cargo run --dev-validator N");
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::log::Builder::from_env(utils::log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let args = std::env::args().collect::<Vec<_>>();
    let mut validator_index = 0;

    match args[1].as_ref() { 
        "--dev-validator" => {
            validator_index = args[2].parse().expect("Error parsing --dev-validator index");
            println!("Validator index: {validator_index}");
        },
        _ => {
            println!("Error: Unknown argument '{}'", args[1]);
            print_help();
            return Ok(());
        },
    };

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| format!("Failed to install ring provider: {:?}", e))?;

    let (certs, key_der) = net_utils::load_credentials(validator_index)?;

    let server_config = net_utils::load_server_config(certs.clone(), key_der.clone_key())?;
    let client_config = net_utils::load_client_config(certs, key_der)?;

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000 + validator_index);
    utils::log::debug!("Listening on {}", bind_addr);
    let mut endpoint = quinn::Endpoint::server(server_config, bind_addr)?;
    endpoint.set_default_client_config(client_config);
    
    let net = std::sync::Arc::new(net_controller::NetworkController::new(endpoint));
    
    let net_for_server = net.clone();
    node_config::set_account_id(validator_index);
    
    let server_handler = tokio::spawn(async move {
        if let Err(e) = net_for_server.listen_network().await {
            log::error!("Server task failed: {:?}", e);
        }
    });

    let identities = dev_accounts::parse_dev_accounts();
    let ed25519_public: Vec<_> = identities.iter().map(|key| key.ed25519_public).collect();
    let mut clients_handler = vec![];

    for (index, ed25519_key) in ed25519_public.iter().enumerate() {
        
        if index == validator_index as usize {
            continue;
        }

        if net_utils::am_i_the_preferred_initiator(&ed25519_public[validator_index as usize], ed25519_key) {
            utils::log::info!("Initialize connection to node {:?}", index);
            let net_for_client = net.clone();
            clients_handler.push(tokio::spawn(async move {
                if let Err(e) = net_for_client.connect_to_peer(index as ValidatorIndex).await {
                    log::error!("Client {} failed: {:?}", index, e);
                }
            }));
        }
    }
    
    if let Err(e) = server_handler.await {
        utils::log::error!("Server join error: {:?}", e);
    }

    for handle in clients_handler {
        if let Err(e) = handle.await {
            utils::log::error!("Client join error: {:?}", e);
        }
    }

    utils::log::info!("End networking");
    
    Ok(())
}
