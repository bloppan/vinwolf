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
    println!("cargo run --dev-validator N");
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    log::Builder::from_env(log::Env::default().default_filter_or("debug"))
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
    log::debug!("Listening on {}", bind_addr);
    let mut endpoint = quinn::Endpoint::server(server_config, bind_addr)?;
    endpoint.set_default_client_config(client_config);
    
    let net = std::sync::Arc::new(net_ctrl::NetworkController::new(endpoint));

    let block_queue_rx = message::init_block_queue(32);
    tokio::spawn(async move {
        message::run_block_queue(block_queue_rx).await;
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

    use network::jamnp_types::TicketDistributed;
    use codec::Encode;
    use time;

    let current_slot = time::current_slot().unwrap();
    time::wait_until_next_slot(current_slot + 2).await.unwrap();

    let identities = dev_accounts::parse_dev_accounts();
    let this_node_index = node_config::get_account_id();
    let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;
    let bandersnatch_secret_seed = identities[this_node_index as usize].bandersnatch_secret_seed;
    
    let curr_validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.curr_validators.clone()
    };

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
            log::info!("New epoch {} detected, generating tickets in background", current_epoch);

            let next_epoch = current_epoch + 1;
            let node_index = this_node_index;

            tokio::spawn(async move {

                let this_node_index = node_config::get_account_id();
                let identities = dev_accounts::parse_dev_accounts();
                let bandersnatch_secret_key = identities[this_node_index as usize].bandersnatch_secret_seed;
                let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;
                
                let tickets = tokio::task::spawn_blocking(move || 
                    block::extrinsic::tickets::generate(bandersnatch_secret_key, bandersnatch_public))
                        .await
                        .expect("ticket generation task panicked");

                let next_validators = {
                    let state = state_handler::get_global_state().lock().unwrap();
                    state.next_validators.clone()
                };

                for (ticket, ticket_id) in tickets {
                    let proxy_index = compute_proxy_index(&ticket_id);
                    log::info!("Ticket attempt={} proxy_index={}", ticket.attempt, proxy_index);

                    let distributed = TicketDistributed { epoch: next_epoch, ticket: ticket.clone() };
                    let blob = distributed.encode();

                    if proxy_index == node_index as usize {
                        block::extrinsic::tickets::store(ticket);
                        tokio::spawn(async move {
                            message::broadcast_ticket_to_validators(blob).await;
                        });
                    } else {
                        let proxy_bandersnatch = next_validators.list[proxy_index].bandersnatch;
                        tokio::spawn(async move {
                            message::send_ticket_to_proxy(blob, &proxy_bandersnatch).await;
                        });
                    }
                }
            });
        }

        if let Some(header) = should_produce_block(current_slot, curr_prover.as_ref().unwrap(), &bandersnatch_public).await {
            let announcement = build_announcement(header);
            net.broadcast_announcement(announcement).await;
        }

        time::wait_until_next_slot(current_slot).await.unwrap();
    }
}

async fn should_produce_block(
    current_slot: TimeSlot, 
    prover: &bandersnatch_vrf_spec::Prover, 
    our_bandersnatch_public: &BandersnatchPublic
) -> Option<Header> {

    let state: GlobalState = {
        let state = state_handler::get_global_state().lock().unwrap().clone();
        state
    };

    let Some(seal) = block::header::get_seal(&state, current_slot) else {
        return None;
    };

    if block::header::seal_winning_verify(&state, seal, current_slot, prover, our_bandersnatch_public) {
        return produce_block(prover).await;
    }

    return None;
}

async fn produce_block(prover: &bandersnatch_vrf_spec::Prover) -> Option<Header> {

    use time;

    let current_slot = time::current_slot().unwrap();

    let state = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.clone()
    };

    let verifier = safrole::verifier::get(ValidatorSet::Pending);

    let block = block::build(&state, current_slot, verifier, prover);

    log::info!("PRODUCING BLOCK...");

    match message::enqueue_block_and_wait(block.clone()).await {
        Ok(_) => {
            message::mark_slot_seen(block.header.unsigned.slot);
            Some(block.header)
        }
        Err(e) => {
            log::error!("STF failed for own block: {:?}", e);
            None
        }
    }
}

fn build_announcement(header: Header) -> Vec<u8> {
    
    use codec::Encode;

    let parent_hash = block::header::get_parent_header();
    let time = state_handler::time::get();

    let announcement = jamnp_types::Announcement {
        header,
        last_finalized_block: jamnp_types::LastFinalizedBlock {
            header_hash: parent_hash,
            slot: time,
        },
    };

    announcement.encode()
}

/// Compute the proxy validator index for a ticket.
/// The proxy is determined by interpreting the last 4 bytes of the ticket's VRF output
/// as a big-endian unsigned integer, modulo the number of validators.
fn compute_proxy_index(ticket_id: &OpaqueHash) -> usize {
    let last_4 = &ticket_id[28..32];
    let val = u32::from_be_bytes(last_4.try_into().unwrap());
    (val as usize) % constants::node::VALIDATORS_COUNT
}

