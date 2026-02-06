pub mod dev_accounts;
pub mod message;
pub mod net_controller;
pub mod net_utils;
pub mod jamnp_codec;
pub mod jamnp_types;

use block::header;
use constants::node::*;
use jam_types::*;
use network::node_config;
use state_handler::time::get_current;
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


async fn node_ctrl(net: std::sync::Arc<net_controller::NetworkController>) {

    let identities = dev_accounts::parse_dev_accounts();
    let this_node_index = node_config::get_account_id();
    let bandersnatch_public = identities[this_node_index as usize].bandersnatch_public;

    loop {

        wait_until_next_slot();
        let slot = current_slot();
        log::debug!("Slot {} started - checking block production", slot);
        if let Some(header) = should_produce_block(slot, &bandersnatch_public) {
            let announcement = build_announcement(header);
            net.broadcast_announcement(announcement).await;
        }
    }
}

fn should_produce_block(slot: TimeSlot, bandersnatch_public: &BandersnatchPublic) -> Option<Header> {

    let safrole_state = state_handler::get_global_state()
        .lock()
        .unwrap()
        .safrole.clone();

    let id = match safrole_state.seal {
        jam_types::Seal::Tickets(ticket) => {
            log::info!("SON TICKETS");
            for idx in 0..EPOCH_LENGTH {
                log::info!("{}", hex::encode(&ticket.tickets_mark[idx].id));
            }
            ticket.tickets_mark[(slot % EPOCH_LENGTH as TimeSlot) as usize].id
        },
        jam_types::Seal::Keys(key) => {
            log::info!("SON KEYS");
            for idx in 0..EPOCH_LENGTH {
                log::info!("{}", hex::encode(&key.epoch[idx]));
            }
            key.epoch[(slot % EPOCH_LENGTH as TimeSlot) as usize]
        },
        _ => { return None; },
    };

    log::info!("KEY id: {}", hex::encode(&id));

    if id == *bandersnatch_public {
        log::info!("WEEEEEEEEEEEEEE AHORA ME TOCA!!!!");
        return Some(produce_block());
    }

    None
}

fn produce_block() -> Header {

    use bandersnatch_vrf_spec::Prover;
    use ark_vrf::reexports::ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_vrf::suites::bandersnatch::{Public, Secret, RingProofParams};
    use codec::Encode;

    let this_node_index = node_config::get_account_id();

    let bandersnatch_public_keys: Vec<BandersnatchPublic> = dev_accounts::parse_dev_accounts()
        .iter()
        .map(|key| key.bandersnatch_public)
        .collect();
    
    let bandersnatch_secret_key: BandersnatchPublic = dev_accounts::parse_dev_accounts()[this_node_index as usize].bandersnatch_secret_seed;

    let mut ring: Vec<Public> = Vec::new();
    for key in bandersnatch_public_keys.iter() {
        let item = Public::deserialize_compressed_unchecked(&key[..])
            .unwrap_or_else(|_| Public::from(RingProofParams::padding_point()));
        ring.push(item);
    }

    let prover_secret = Secret::from_seed(&bandersnatch_secret_key);

    let prover = Prover {
        prover_idx: this_node_index as usize,
        secret: prover_secret,
        ring: ring.clone(),
    };

    let entropy = state_handler::get_global_state()
        .lock()
        .unwrap()
        .entropy
        .clone();

    // Entropy source 


    let mut block = Block::default();
    block.header.unsigned.author_index = this_node_index;
    block.header.unsigned.extrinsic_hash = header::encode_extrinsic(&block);
    block.header.unsigned.parent = block::header::get_parent_header();
    block.header.unsigned.parent_state_root = state_handler::get_state_root().lock().unwrap().clone();
    block.header.unsigned.slot = current_slot();

    // Seal

    // Step 1
    let c = [&b"jam_fallback_seal"[..], &entropy.buf[3].encode()].concat();
    let vrf_output = prover.vrf_output(&c);

    // Step 2 
    let context = [&b"jam_entropy"[..], &vrf_output.encode()].concat();
    block.header.unsigned.entropy_source = prover.ietf_vrf_sign(&context, &[]).try_into().unwrap();

    // Step 3
    block.header.seal = prover.ietf_vrf_sign(&c, &block.header.unsigned.encode()).try_into().unwrap();

    message::store_block(&block);
    let _ = state_controller::stf(&block);

    block.header
}

fn build_announcement(header: Header) -> Vec<u8> {
    use crate::jamnp_types::{Announcement, LastFinalizedBlock};
    use codec::Encode;

    let parent_hash = block::header::get_parent_header();
    let time = state_handler::time::get();

    let announcement = Announcement {
        header,
        last_finalized_block: LastFinalizedBlock {
            header_hash: parent_hash,
            slot: time,
        },
    };

    announcement.encode()
}

fn wait_until_next_slot() {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards");

    let now_millis = now.as_millis() as u64;
    let era_millis = constants::node::JAM_COMMON_ERA * 1000;
    let slot_millis = (constants::node::SLOT_PERIOD as u64) * 1000;

    // Calculate milliseconds elapsed since JAM era
    let elapsed_since_era = now_millis.saturating_sub(era_millis);

    // Calculate how many milliseconds into the current slot we are
    let millis_into_slot = elapsed_since_era % slot_millis;

    // Calculate milliseconds until next slot starts
    let millis_until_next_slot = slot_millis - millis_into_slot;

    // Sleep until the next slot boundary
    std::thread::sleep(Duration::from_millis(millis_until_next_slot));
}

pub fn current_slot() -> jam_types::TimeSlot {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs();

    ((now_secs - constants::node::JAM_COMMON_ERA) as TimeSlot / constants::node::SLOT_PERIOD) as TimeSlot
}

//fn evaluate_block_production

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn node_ctrl_test() {
        node_ctrl();
    }

    #[test]
    fn wait_until_next_slot_test() {
        let start = std::time::Instant::now();
        wait_until_next_slot();
        let slot = current_slot();
        let elapsed = start.elapsed();

        println!("Waited {:?} for slot {}", elapsed, slot);

        // Verify we're at the beginning of the slot (within 50ms tolerance)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let era_millis = constants::node::JAM_COMMON_ERA * 1000;
        let slot_millis = (constants::node::SLOT_PERIOD as u64) * 1000;
        let elapsed_since_era = now.as_millis() as u64 - era_millis;
        let millis_into_slot = elapsed_since_era % slot_millis;

        println!("Milliseconds into slot: {}", millis_into_slot);
        assert!(millis_into_slot < 50, "Should be at start of slot");
    }
}

