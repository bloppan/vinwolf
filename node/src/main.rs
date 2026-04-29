use codec::Encode;
use constants::node::*;
use jam_types::*;
use network::{dev_accounts, grid, jamnp_types, message, node_config, NetworkController};
use std::error::Error;
use std::sync::Arc;
use tools::{log, rpc::RpcServer};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

struct Config {
    validator_index: ValidatorIndex,
    rpc_port: u16,
}

fn print_help() {
    println!("vinwolf node");
    println!();
    println!("\x1b[1mUsage example:\x1b[0m\n");
    println!("cargo run -p node -- --dev-validator N [--rpc-port PORT]");
    println!();
    println!("  --dev-validator N   validator index (default: 0)");
    println!("  --rpc-port PORT     RPC server port to connect to (default: 19800)");
    println!();
}

fn parse_args() -> Result<Option<Config>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut validator_index: ValidatorIndex = 0;
    let mut rpc_port: u16 = 19800;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dev-validator" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "--dev-validator requires a value".to_string())?;
                validator_index = value
                    .parse()
                    .map_err(|e| format!("Error parsing --dev-validator index: {e}"))?;
                log::debug!("Validator index: {}", validator_index);
            }
            "--rpc-port" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "--rpc-port requires a value".to_string())?;
                rpc_port = value
                    .parse()
                    .map_err(|e| format!("Error parsing --rpc-port: {e}"))?;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(None);
            }
            arg => {
                println!("Error: Unknown argument '{}'", arg);
                print_help();
                return Ok(None);
            }
        }
        i += 1;
    }

    Ok(Some(Config {
        validator_index,
        rpc_port,
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    log::Builder::from_env(log::Env::default().default_filter_or("debug"))
        .with_dotenv(true)
        .init();

    let Some(config) = parse_args()? else {
        return Ok(());
    };

    let rpc_server = RpcServer::bind(config.rpc_port)
        .map_err(|e| format!("Failed to bind RPC server: {}", e))?;
    log::info!("RPC server listening on port {}", config.rpc_port);

    std::thread::spawn(move || {
        listen_rpc(rpc_server);
    });

    let network = network::init_network(config.validator_index).await?;

    let net_for_node = network.controller.clone();
    let node_ctrl_handle = tokio::spawn(async move { 
        node_ctrl(net_for_node).await 
    });

    match node_ctrl_handle.await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => log::error!("Node ctrl failed: {:?}", e),
        Err(e) => log::error!("Node ctrl join error: {:?}", e),
    }

    if let Err(e) = network.server_handle.await {
        log::error!("Server join error: {:?}", e);
    }

    log::info!("End node");

    Ok(())
}

async fn node_ctrl(net: Arc<NetworkController>) -> Result<()> {
    let current_slot =
        time::current_slot().map_err(|e| format!("Failed to get current slot: {:?}", e))?;
    time::wait_until_next_slot(current_slot + 2)
        .await
        .map_err(|e| format!("Failed while waiting for initial slot: {:?}", e))?;

    let this_node_index = node_config::get_account_id();
    let identities = dev_accounts::parse_dev_accounts();
    let identity = identities
        .get(this_node_index as usize)
        .ok_or_else(|| format!("Validator index {} is out of range", this_node_index))?;
    let bandersnatch_public = identity.bandersnatch_public;
    let bandersnatch_secret_seed = identity.bandersnatch_secret_seed;

    let curr_validators = state_handler::get_global_state()
        .lock()
        .map_err(|e| format!("Global state lock poisoned: {e}"))?
        .curr_validators
        .clone();

    let mut cached_epoch: Option<TimeSlot> = None;
    let mut curr_prover: Option<bandersnatch_vrf_spec::Prover> = None;

    loop {
        let current_slot =
            time::current_slot().map_err(|e| format!("Failed to get current slot: {:?}", e))?;
        let current_epoch = current_slot / EPOCH_LENGTH as TimeSlot;
        log::debug!(
            "Slot {} started (epoch {}) - checking block production",
            current_slot,
            current_epoch
        );

        // Rebuild prover only when epoch changes.
        if cached_epoch != Some(current_epoch) {
            log::info!("Building prover for epoch {}", current_epoch);
            curr_prover = Some(safrole::build_curr_prover(
                &curr_validators,
                &bandersnatch_public,
                bandersnatch_secret_seed,
            ));
            cached_epoch = Some(current_epoch);
        }

        if current_slot % EPOCH_LENGTH as TimeSlot == 3 {
            spawn_ticket_generation(
                current_epoch,
                this_node_index,
                bandersnatch_secret_seed,
                bandersnatch_public,
            );
        }

        let state = state_handler::get_global_state()
            .lock()
            .map_err(|e| format!("Global state lock poisoned: {e}"))?
            .clone();

        let Some(prover) = curr_prover.as_ref() else {
            return Err("Prover was not initialized".into());
        };

        if let Some(seal) = block::header::get_seal(&state, current_slot) {
            if block::header::seal_winning_verify(
                &state,
                seal,
                current_slot,
                prover,
                &bandersnatch_public,
            ) {
                let verifier = safrole::verifier::get(ValidatorSet::Pending);

                log::info!("PRODUCING BLOCK...");
                let block = block::build(&state, current_slot, verifier, prover);

                match message::block::enqueue_and_wait(block.clone()).await {
                    Ok(_) => {
                        message::block::mark_slot_seen(block.header.unsigned.slot);
                        let announcement =
                            Arc::<[u8]>::from(message::block::announcement::build(block.header));
                        message::block::announcement::broadcast(net.as_ref(), announcement).await;
                    }
                    Err(e) => {
                        log::error!("STF failed for own block: {:?}", e);
                    }
                }
            }
        }

        time::wait_until_next_slot(current_slot)
            .await
            .map_err(|e| format!("Failed while waiting for next slot: {:?}", e))?;
    }
}

fn spawn_ticket_generation(
    current_epoch: TimeSlot,
    node_index: ValidatorIndex,
    bandersnatch_secret_seed: BandersnatchSecret,
    bandersnatch_public: BandersnatchPublic,
) {
    let next_epoch = current_epoch + 1;
    log::info!(
        "New epoch {} detected, generating tickets in background",
        current_epoch
    );

    tokio::spawn(async move {
        let tickets = match tokio::task::spawn_blocking(move || {
            block::extrinsic::tickets::generate(bandersnatch_secret_seed, bandersnatch_public)
        })
        .await
        {
            Ok(tickets) => tickets,
            Err(e) => {
                log::error!("Ticket generation task failed: {:?}", e);
                return;
            }
        };

        let next_validators = match state_handler::get_global_state().lock() {
            Ok(state) => state.next_validators.clone(),
            Err(e) => {
                log::error!("Global state lock poisoned during ticket generation: {}", e);
                return;
            }
        };

        for (ticket, ticket_id) in tickets {
            let proxy_index = grid::compute_proxy_index(&ticket_id);
            log::info!(
                "Ticket attempt={} proxy_index={}",
                ticket.attempt,
                proxy_index
            );

            let distributed = jamnp_types::TicketDistributed {
                epoch: next_epoch,
                ticket: ticket.clone(),
            };
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

fn listen_rpc(rpc_server: RpcServer) {
    rpc_server.run(|method, _params| match method {
        "bestBlock" => {
            let state = state_handler::get_global_state().lock().map_err(|e| {
                (
                    tools::rpc::codes::CODE_INTERNAL_ERROR,
                    format!("Global state lock poisoned: {}", e),
                )
            })?;
            let header_hash = block::header::get_parent_header();
            let mut m = std::collections::HashMap::new();
            m.insert(
                "slot".into(),
                tools::serde::Value::Number(state.time.to_string()),
            );
            m.insert(
                "header_hash".into(),
                tools::serde::Value::String(tools::hex::encode(header_hash)),
            );
            Ok(tools::serde::Value::Object(m))
        }
        _ => Err((
            tools::rpc::codes::CODE_METHOD_NOT_FOUND,
            format!("method not found: {}", method),
        )),
    });
}
