pub mod client;
pub mod dev_accounts;
pub mod message;
pub mod net_utils;
pub mod jamnp_codec;
pub mod jamnp_types;
pub mod server;

use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use jam_types::{Ed25519Public, ValidatorIndex};
use network::node_config;
use utils::log;

use crate::client::run_client;
use crate::server::run_server;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn print_help() {    
    println!("vinwolf network");
    println!();
    println!("\x1b[1mUsage example:\x1b[0m\n");
    println!("cargo run --dev-validator N");
    println!();
}

fn am_i_the_preferred_initiator(my_key: &Ed25519Public, peer_key: &Ed25519Public) -> bool {
    let cond = ((my_key[31] > 127) ^ (peer_key[31] > 127)) ^ (my_key < peer_key);
    cond
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

    let port = 40000 + validator_index;
    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let server_config = net_utils::load_server_config(validator_index)?;
    
    let mut endpoint = quinn::Endpoint::server(server_config.clone(), bind_addr)?;
    utils::log::debug!("Listening on {}", bind_addr);

    let client_config = net_utils::load_client_config(validator_index)?;
    endpoint.set_default_client_config(client_config);

    node_config::set_account_id(validator_index);

    let server_endpoint = endpoint.clone();
    let server_handler = tokio::spawn(async move {
        if let Err(e) = run_server(validator_index, server_endpoint).await {
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

        if am_i_the_preferred_initiator(&ed25519_public[validator_index as usize], ed25519_key) {
            utils::log::info!("Initialize connection to node {:?}", index);
            let client_endpoint = endpoint.clone();
            clients_handler.push(tokio::spawn(async move {
                if let Err(e) = run_client(client_endpoint).await {
                    utils::log::error!("Client task for node {} failed: {:?}", index, e);
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
