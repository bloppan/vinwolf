use ::block::extrinsic;
use crate::{dev_accounts, node_config};
use crate::jamnp_types::*;
use crate::message::{NetworkMessage, TICKET_GENERATION, TICKET_PROXY};
use codec::Encode;
use codec::generic_codec::decode_from_bytes;
use jam_types::{*};
use quinn::RecvStream;
use tools::{hex, log};

pub struct TicketDistribution {
    pub epoch_index: TimeSlot,
    pub ticket: Ticket,
}

pub async fn send_to_proxy(distributed_ticket_blob: Vec<u8>, proxy_bandersnatch: &BandersnatchPublic) {

    let connection = match dev_accounts::get_dev_account_connection(proxy_bandersnatch) {
        Some(conn) => conn,
        None => {
            log::error!("No connection to proxy validator {}", hex::encode(proxy_bandersnatch));
            return;
        }
    };

    let (mut send_stream, _recv_stream) = match connection.open_bi().await {
        Ok(streams) => streams,
        Err(e) => {
            log::error!("Failed to open stream to proxy validator: {:?}", e);
            return;
        }
    };

    log::info!("Sending ticket to proxy validator at {:?} (CE 131)", connection.remote_address());
    if let Err(e) = NetworkMessage::ce_send(TICKET_GENERATION, distributed_ticket_blob, &mut send_stream).await {
        log::error!("Failed to send ticket to proxy: {:?}", e);
    }
}

pub async fn recv_from_generator(mut recv_stream: RecvStream) -> Result<(), NetworkError> {

    let distributed_ticket_blob = NetworkMessage::recv(&mut recv_stream).await?;

    let distributed_ticket = decode_from_bytes::<TicketDistributed>(&distributed_ticket_blob).map_err(|e| {
        log::error!("Failed to decode distributed ticket: {:?}", e);
        NetworkError::Decode(e)
    })?;

    log::debug!("Ticket received (CE 131): attempt={} epoch={}",
        distributed_ticket.ticket.attempt, distributed_ticket.epoch);

    // Verify that we are the correct proxy for this ticket
    let entropy_state = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.entropy.clone()
    };

    let verifier = safrole::verifier::get(ValidatorSet::Next);
    let fixed_input_data: Vec<u8> = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode()].concat();
    let vrf_input = [&fixed_input_data[..], &distributed_ticket.ticket.attempt.encode()].concat();

    let ticket_id = match verifier.ring_vrf_verify(&vrf_input, &[], &distributed_ticket.ticket.signature) {
        Ok(id) => id,
        Err(_) => {
            log::error!("Invalid ticket proof received on CE 131, discarding");
            return Ok(());
        }
    };

    let proxy_index = {
        let last_4 = &ticket_id[28..32];
        let val = u32::from_be_bytes(last_4.try_into().unwrap());
        (val as usize) % constants::node::VALIDATORS_COUNT
    };

    let this_node = node_config::get_account_id() as usize;
    if proxy_index != this_node {
        log::error!("We are not the correct proxy for this ticket (proxy={}, we={}), discarding", proxy_index, this_node);
        return Ok(());
    }

    log::info!("We are the correct proxy for ticket attempt={}, forwarding to all validators (CE 132)",
        distributed_ticket.ticket.attempt);

    extrinsic::tickets::store(distributed_ticket.ticket);

    tokio::spawn(async move {
        broadcast_to_validators(distributed_ticket_blob).await;
    });

    Ok(())
}

pub async fn broadcast_to_validators(distributed_ticket_blob: Vec<u8>) {

    let validators = {
        let state = state_handler::get_global_state().lock().unwrap();
        state.curr_validators.list.clone()
    };

    let this_node = node_config::get_account_id();

    for (i, validator) in validators.iter().enumerate() {

        if i == this_node as usize {
            continue;
        }

        let connection = match dev_accounts::get_dev_account_connection(&validator.bandersnatch) {
            Some(conn) => conn,
            None => {
                log::error!("Getting dev account connection for validator: {i}");
                continue;
            }
        };

        let (mut send_stream, _recv_stream) = match connection.open_bi().await {
            Ok(streams) => streams,
            Err(e) => {
                log::error!("Failed to open stream to validator {i}: {:?}", e);
                continue;
            }
        };

        log::info!("Proxy ticket to validator {i} at {:?}", connection.remote_address());
        if let Err(e) = NetworkMessage::ce_send(TICKET_PROXY, distributed_ticket_blob.clone(), &mut send_stream).await {
            log::error!("Failed to send ticket to validator {i}: {:?}", e);
        }
    }
}

pub async fn recv_distribution(mut recv_stream: RecvStream) -> Result<(), NetworkError> {

    let distributed_ticket_blob = NetworkMessage::recv(&mut recv_stream).await?;

    let distributed_ticket = decode_from_bytes::<TicketDistributed>(&distributed_ticket_blob).map_err(|e| {
        log::error!("Failed to decode distributed ticket: {:?}", e);
        NetworkError::Decode(e)
    })?;

    log::info!("Ticket distribution received: epoch={} attempt={}", distributed_ticket.epoch, distributed_ticket.ticket.attempt);
    extrinsic::tickets::store(distributed_ticket.ticket);

    Ok(())
}