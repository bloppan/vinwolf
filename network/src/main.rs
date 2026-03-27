use codec::Encode;
use constants::node::*;
use jam_types::*;
use network::{dev_accounts, jamnp_types, message, net_ctrl, net_utils, node_config, grid};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tools::log;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

fn print_help() {
    println!("vinwolf network");
    println!();
    println!("\x1b[1mUsage example:\x1b[0m\n");
    println!("cargo run --dev-validator N [--rpc-port PORT]");
    println!();
    println!("  --dev-validator N   validator index (required)");
    println!("  --rpc-port PORT     RPC server port to connect to (default: 19800)");
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {

    log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let args = std::env::args().collect::<Vec<_>>();
    let mut validator_index: u16 = 0;
    let mut rpc_port: u16 = 19800;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dev-validator" => {
                i += 1;
                validator_index = args.get(i)
                    .expect("--dev-validator requires a value")
                    .parse()
                    .expect("Error parsing --dev-validator index");
                log::debug!("Validator index: {}", validator_index);
            }
            "--rpc-port" => {
                i += 1;
                rpc_port = args.get(i)
                    .expect("--rpc-port requires a value")
                    .parse()
                    .expect("Error parsing --rpc-port");
            }
            arg => {
                println!("Error: Unknown argument '{}'", arg);
                print_help();
                return Ok(());
            }
        }
        i += 1;
    }

    let rpc_server = tools::rpc::RpcServer::bind(rpc_port)
        .map_err(|e| format!("Failed to bind RPC server: {}", e))?;
    log::info!("RPC server listening on port {}", rpc_port);

    std::thread::spawn(move || {
        rpc_server.run(|method, _params| {
            match method {
                "bestBlock" => {
                    let state = state_handler::get_global_state().lock().unwrap();
                    let mut m = std::collections::HashMap::new();
                    m.insert("slot".into(), tools::serde::Value::Number(state.time.to_string()));
                    m.insert("header_hash".into(), tools::serde::Value::String("hardcoded".into()));
                    Ok(tools::serde::Value::Object(m))
                }
                _ => Err((tools::rpc::codes::CODE_METHOD_NOT_FOUND, format!("method not found: {}", method))),
            }
        });
    });

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
    
    let net = std::sync::Arc::new(net_ctrl::NetworkController::new(endpoint));

    let block_queue_rx = message::block::init_queue(32);
    tokio::spawn(async move {
        message::block::run_queue(block_queue_rx).await;
    });

    node_config::set_account_id(validator_index);

    let identities = dev_accounts::parse_dev_accounts();
    let ed25519_public: Vec<_> = identities.iter().map(|key| key.ed25519_public).collect();

    net.init_peers(validator_index, &ed25519_public).await;

    let net_for_server = net.clone();
    let server_handler = tokio::spawn(async move {
        if let Err(e) = net_for_server.listen_network().await {
            log::error!("Server task failed: {:?}", e);
        }
    });

    // Initial connection attempts (fire-and-forget, monitor will reconnect if they fail)
    for (index, ed25519_key) in ed25519_public.iter().enumerate() {
        
        if index == validator_index as usize {
            continue;
        }

        if grid::am_i_the_preferred_initiator(&ed25519_public[validator_index as usize], ed25519_key) {
            log::info!("Initialize connection to node {:?}", index);
            let net_for_client = net.clone();
            tokio::spawn(async move {
                if let Err(e) = net_for_client.connect_to_peer(index as ValidatorIndex).await {
                    log::error!("Client {} failed: {:?}", index, e);
                }
            });
        }
    }

    let net_for_monitor = net.clone();
    tokio::spawn(async move {
        net_for_monitor.connection_monitor().await;
    });

    let net_for_node = net.clone();
    let node_ctrl_handle = tokio::spawn(async move {
        node_ctrl(net_for_node).await;
    });

    if let Err(e) = node_ctrl_handle.await {
        log::error!("Node ctrl join error: {:?}", e);
    }

    if let Err(e) = server_handler.await {
        log::error!("Server join error: {:?}", e);
    }

    log::info!("End networking");
    
    Ok(())
}

async fn node_ctrl(net: std::sync::Arc<net_ctrl::NetworkController>) {
    let current_slot = time::current_slot().unwrap();
    time::wait_until_next_slot(current_slot + 2).await.unwrap();

    let this_node_index = node_config::get_account_id();
    let identities = dev_accounts::parse_dev_accounts();
    let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;
    let bandersnatch_secret_seed = identities[this_node_index as usize].bandersnatch_secret_seed;

    let curr_validators = state_handler::get_global_state().lock().unwrap().curr_validators.clone();

    let mut cached_epoch: Option<TimeSlot> = None;
    let mut curr_prover: Option<bandersnatch_vrf_spec::Prover> = None;

    loop {
        let current_slot = time::current_slot().unwrap();
        let current_epoch = current_slot / EPOCH_LENGTH as TimeSlot;
        log::debug!("Slot {} started (epoch {}) - checking block production", current_slot, current_epoch);

        // Rebuild prover only when epoch changes
        if cached_epoch != Some(current_epoch) {
            log::info!("Building prover for epoch {}", current_epoch);
            curr_prover = Some(safrole::build_curr_prover(&curr_validators, &bandersnatch_public, bandersnatch_secret_seed));
            cached_epoch = Some(current_epoch);
        }

        if current_slot % EPOCH_LENGTH as TimeSlot == 3 {
            spawn_ticket_generation(current_epoch, this_node_index, bandersnatch_secret_seed, bandersnatch_public);
        }

        let state = {
            state_handler::get_global_state().lock().unwrap().clone()
        };

        let prover = curr_prover.as_ref().unwrap();

        if let Some(seal) = block::header::get_seal(&state, current_slot) {
            if block::header::seal_winning_verify(&state, seal, current_slot, prover, &bandersnatch_public) {
                let verifier = safrole::verifier::get(ValidatorSet::Pending);

                log::info!("PRODUCING BLOCK...");
                let block = block::build(&state, current_slot, verifier, prover);

                match message::block::enqueue_and_wait(block.clone()).await {
                    Ok(_) => {
                        message::block::mark_slot_seen(block.header.unsigned.slot);
                        let announcement = message::block::build_announcement(block.header);
                        net.broadcast_announcement(announcement).await;
                    }
                    Err(e) => {
                        log::error!("STF failed for own block: {:?}", e);
                    }
                }
            }
        }

        time::wait_until_next_slot(current_slot).await.unwrap();
    }
}

fn spawn_ticket_generation(
    current_epoch: TimeSlot,
    node_index: ValidatorIndex,
    bandersnatch_secret_seed: BandersnatchSecret,
    bandersnatch_public: BandersnatchPublic,
) {
    let next_epoch = current_epoch + 1;
    log::info!("New epoch {} detected, generating tickets in background", current_epoch);

    tokio::spawn(async move {
        let tickets = tokio::task::spawn_blocking(move ||
            block::extrinsic::tickets::generate(bandersnatch_secret_seed, bandersnatch_public))
                .await
                .expect("ticket generation task panicked");

        let next_validators = {
            state_handler::get_global_state().lock().unwrap().next_validators.clone()
        };

        for (ticket, ticket_id) in tickets {
            let proxy_index = grid::compute_proxy_index(&ticket_id);
            log::info!("Ticket attempt={} proxy_index={}", ticket.attempt, proxy_index);

            let distributed = jamnp_types::TicketDistributed { epoch: next_epoch, ticket: ticket.clone() };
            let blob = distributed.encode();

            if proxy_index == node_index as usize {
                block::extrinsic::tickets::store(ticket);
                tokio::spawn(async move {
                    message::ticket::broadcast_to_validators(blob).await;
                });
            } else {
                let proxy_bandersnatch = next_validators.list[proxy_index].bandersnatch;
                tokio::spawn(async move {
                    message::ticket::send_to_proxy(blob, &proxy_bandersnatch).await;
                });
            }
        }
    });
}
