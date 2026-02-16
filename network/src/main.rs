use block::header;
use constants::node::*;
use jam_types::*;
use network::{dev_accounts, jamnp_types, message, net_ctrl, net_utils, node_config, topology};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tools::{hex, log};

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

        if topology::am_i_the_preferred_initiator(&ed25519_public[validator_index as usize], ed25519_key) {
            log::info!("Initialize connection to node {:?}", index);
            let net_for_client = net.clone();
            clients_handler.push(tokio::spawn(async move {
                if let Err(e) = net_for_client.connect_to_peer(index as ValidatorIndex).await {
                    log::error!("Client {} failed: {:?}", index, e);
                }
            }));
        }
    }
    
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

    for handle in clients_handler {
        if let Err(e) = handle.await {
            log::error!("Client join error: {:?}", e);
        }
    }

    log::info!("End networking");
    
    Ok(())
}


fn build_curr_prover(bandersnatch_public: &BandersnatchPublic) -> bandersnatch_vrf_spec::Prover {
    use ark_vrf::reexports::ark_serialize::CanonicalDeserialize;
    use ark_vrf::suites::bandersnatch::{Public, Secret, RingProofParams};

    let this_node_index = node_config::get_account_id();
    let identities = dev_accounts::parse_dev_accounts();
    let bandersnatch_secret_seed = identities[this_node_index as usize].bandersnatch_secret_seed;

    let curr_validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.curr_validators.clone()
    };

    let ring: Vec<Public> = curr_validators.list.iter()
        .map(|v| Public::deserialize_compressed_unchecked(&v.bandersnatch[..])
            .unwrap_or_else(|_| Public::from(RingProofParams::padding_point())))
        .collect();

    let prover_idx = curr_validators.list.iter()
        .position(|v| v.bandersnatch == *bandersnatch_public)
        .expect("This validator must be present in curr_validators");

    bandersnatch_vrf_spec::Prover {
        prover_idx,
        secret: Secret::from_seed(&bandersnatch_secret_seed),
        ring,
    }
}

async fn node_ctrl(net: std::sync::Arc<net_ctrl::NetworkController>) {

    use network::jamnp_types::TicketDistributed;
    use codec::Encode;
    use time;

    let slot = time::current_slot().unwrap();
    time::wait_until_next_slot(slot).await.unwrap();

    let identities = dev_accounts::parse_dev_accounts();
    let this_node_index = node_config::get_account_id();
    let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;

    let mut cached_epoch: Option<TimeSlot> = None;
    let mut curr_prover: Option<bandersnatch_vrf_spec::Prover> = None;

    loop {

        let slot = time::current_slot().unwrap();
        let current_epoch = slot / EPOCH_LENGTH as TimeSlot;
        log::debug!("Slot {} started (epoch {}) - checking block production", slot, current_epoch);

        // Rebuild prover only when epoch changes
        if cached_epoch != Some(current_epoch) {
            log::info!("Building prover for epoch {}", current_epoch);
            curr_prover = Some(build_curr_prover(&bandersnatch_public));
            cached_epoch = Some(current_epoch);
        }

        if slot % EPOCH_LENGTH as TimeSlot == 4 {
            log::info!("New epoch {} detected, generating tickets", current_epoch);

            message::clear_ticket_pool();
            let tickets = generate_tickets();
            let next_epoch = current_epoch + 1;

            let next_validators = {
                let state = state_handler::get_global_state().lock().unwrap();
                state.next_validators.clone()
            };

            for (ticket, ticket_id) in tickets {
                let proxy_index = compute_proxy_index(&ticket_id);
                log::info!("Ticket attempt={} proxy_index={}", ticket.attempt, proxy_index);

                let distributed = TicketDistributed { epoch: next_epoch, ticket: ticket.clone() };
                let blob = distributed.encode();

                if proxy_index == this_node_index as usize {
                    // We are the proxy: store locally and broadcast to all current validators (CE 132)
                    message::store_ticket(ticket);
                    tokio::spawn(async move {
                        message::broadcast_ticket_to_validators(blob).await;
                    });
                } else {
                    // Send to the proxy validator via CE 131
                    let proxy_bandersnatch = next_validators.list[proxy_index].bandersnatch;
                    tokio::spawn(async move {
                        message::send_ticket_to_proxy(blob, &proxy_bandersnatch).await;
                    });
                }
            }
        }

        if let Some(header) = should_produce_block(slot, curr_prover.as_ref().unwrap(), &bandersnatch_public).await {
            let announcement = build_announcement(header);
            net.broadcast_announcement(announcement).await;
        }

        time::wait_until_next_slot(slot).await.unwrap();
    }
}

async fn should_produce_block(slot: TimeSlot, prover: &bandersnatch_vrf_spec::Prover, bandersnatch_public: &BandersnatchPublic) -> Option<Header> {

    use codec::Encode;

    let (safrole_state, entropy) = {
        let state = state_handler::get_global_state().lock().unwrap();
        (state.safrole.clone(), state.entropy.clone())
    };

    let slot_index = (slot % EPOCH_LENGTH as TimeSlot) as usize;

    match safrole_state.seal {
        Seal::Tickets(ref tickets) => {
            let ticket = &tickets.tickets_mark[slot_index];
            log::info!("Seal mode: Tickets. Slot {} ticket id: {}", slot, hex::encode(&ticket.id));

            let context = [&b"jam_ticket_seal"[..], &entropy.buf[3].encode(), &ticket.attempt.encode()].concat();
            let our_vrf_output: Vec<u8> = prover.vrf_output(&context);

            if our_vrf_output == ticket.id {
                log::info!("Block production: ticket matches! Slot {}", slot);
                return produce_block(prover).await;
            }
        },
        Seal::Keys(ref keys) => {
            let key = &keys.epoch[slot_index];
            log::info!("Seal mode: Keys (fallback). Slot {} key: {}", slot, hex::encode(key));

            if *key == *bandersnatch_public {
                log::info!("Block production: key matches! Slot {}", slot);
                return produce_block(prover).await;
            }
        },
        Seal::None => {},
    }

    None
}

async fn produce_block(prover: &bandersnatch_vrf_spec::Prover) -> Option<Header> {

    use codec::Encode;
    use time;

    let entropy = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.entropy.clone()
    };

    let mut block = Block::default();
    block.extrinsic.tickets = message::take_tickets_for_block(time::current_slot().unwrap());
    block.header.unsigned.author_index = prover.prover_idx as ValidatorIndex;
    block.header.unsigned.extrinsic_hash = header::encode_extrinsic(&block);
    block.header.unsigned.parent = block::header::get_parent_header();
    block.header.unsigned.parent_state_root = state_handler::get_state_root().lock().unwrap().clone();
    block.header.unsigned.slot = time::current_slot().unwrap();

    let safrole_state = state_handler::get_global_state()
        .lock()
        .unwrap()
        .safrole.clone();

    // Seal
    match safrole_state.seal {
        Seal::Tickets(tickets) => {
            // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
            let slot_index = (block.header.unsigned.slot % EPOCH_LENGTH as TimeSlot) as usize;
            let c = [&b"jam_ticket_seal"[..], &entropy.buf[3].encode(), &tickets.tickets_mark[slot_index].attempt.encode()].concat();
            let vrf_output = prover.vrf_output(&c);
            // Step 2
            let context = [&b"jam_entropy"[..], &vrf_output.encode()].concat();
            block.header.unsigned.entropy_source = prover.ietf_vrf_sign(&context, &[]).try_into().unwrap();
            // Step 3
            block.header.seal = prover.ietf_vrf_sign(&c, &block.header.unsigned.encode()).try_into().unwrap();
        },
        Seal::Keys(keys) => {
            // Step 1
            let c = [&b"jam_fallback_seal"[..], &entropy.buf[3].encode()].concat();
            let vrf_output = prover.vrf_output(&c);
            // Step 2
            let context = [&b"jam_entropy"[..], &vrf_output.encode()].concat();
            block.header.unsigned.entropy_source = prover.ietf_vrf_sign(&context, &[]).try_into().unwrap();
            // Step 3
            block.header.seal = prover.ietf_vrf_sign(&c, &block.header.unsigned.encode()).try_into().unwrap();
        },
        Seal::None => {
            { };
        },
    }

    log::info!("PRODUCING BLOCK...");

    match message::enqueue_block_and_wait(block.clone()).await {
        Ok(_) => {
            let announcement = jamnp_types::Announcement {
                header: block.header.clone(),
                last_finalized_block: jamnp_types::LastFinalizedBlock {
                    header_hash: block.header.unsigned.parent,
                    slot: block.header.unsigned.slot,
                }
            };
            message::set_last_announcement(announcement);
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

fn generate_tickets() -> Vec<(Ticket, OpaqueHash)> {
    use bandersnatch_vrf_spec::Prover;
    use ark_vrf::reexports::ark_serialize::CanonicalDeserialize;
    use ark_vrf::suites::bandersnatch::{Public, RingProofParams};
    use ark_vrf::suites::bandersnatch::Secret;
    use codec::Encode;

    let this_node_index = node_config::get_account_id();
    let identities = dev_accounts::parse_dev_accounts();
    let bandersnatch_secret_key = identities[this_node_index as usize].bandersnatch_secret_seed;
    let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;

    // Build the ring from next_validators (matching the verifier used for verification)
    let next_validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.next_validators.clone()
    };

    let ring: Vec<Public> = next_validators.list.iter()
        .map(|v| Public::deserialize_compressed_unchecked(&v.bandersnatch[..])
            .unwrap_or_else(|_| Public::from(RingProofParams::padding_point())))
        .collect();

    // Find our index within next_validators
    let prover_idx = next_validators.list.iter()
        .position(|v| v.bandersnatch == bandersnatch_public)
        .expect("This validator must be present in next_validators");

    let prover = Prover {
        prover_idx,
        secret: Secret::from_seed(&bandersnatch_secret_key),
        ring,
    };

    let entropy = state_handler::get_global_state()
        .lock()
        .unwrap()
        .entropy
        .clone();

    let fixed_input = [&b"jam_ticket_seal"[..], &entropy.buf[2].encode()].concat();

    let mut tickets = Vec::with_capacity(TICKET_ENTRIES_PER_VALIDATOR as usize);

    for attempt in 0..TICKET_ENTRIES_PER_VALIDATOR {
        let vrf_input = [&fixed_input[..], &attempt.encode()].concat();
        let ticket_id: OpaqueHash = prover.vrf_output(&vrf_input).try_into()
            .expect("vrf_output should be 32 bytes");
        let signature_bytes = prover.ring_vrf_sign(&vrf_input, &[]);
        let signature: BandersnatchRingVrfSignature = signature_bytes.try_into()
            .expect("ring_vrf_sign output should be 784 bytes");

        tickets.push((Ticket { attempt, signature }, ticket_id));
        log::info!("Generated ticket attempt={} id={}", attempt, hex::encode(&ticket_id));
    }

    tickets
}

/// Compute the proxy validator index for a ticket.
/// The proxy is determined by interpreting the last 4 bytes of the ticket's VRF output
/// as a big-endian unsigned integer, modulo the number of validators.
fn compute_proxy_index(ticket_id: &OpaqueHash) -> usize {
    let last_4 = &ticket_id[28..32];
    let val = u32::from_be_bytes(last_4.try_into().unwrap());
    (val as usize) % constants::node::VALIDATORS_COUNT
}

