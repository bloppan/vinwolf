/*
    Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” to determine 
    which validators are privileged to author a block for each timeslot in the following epoch. Tickets specify an entry index 
    together with a proof of ticket’s validity. The proof implies a ticket identifier, a high-entropy unbiasable 32-octet sequence, 
    which is used both as a score in the aforementioned contest and as input to the on-chain vrf. Towards the end of the epoch 
    (i.e. Y slots from the start) this contest is closed implying successive blocks within the same epoch must have an empty tickets 
    extrinsic. At this point, the following epoch’s seal key sequence becomes fixed. We define the extrinsic as a sequence of proofs 
    of valid tickets, each of which is a tuple of an entry index (a natural number less than N) and a proof of ticket validity.
*/

use ark_vrf::reexports::ark_serialize::CanonicalDeserialize;
use ark_vrf::suites::bandersnatch::{Public, RingProofParams};
use ark_vrf::suites::bandersnatch::Secret;
use bandersnatch_vrf_spec::{Prover, Verifier};
use codec::Encode;
use codec::generic_codec::encode_unsigned;
use constants::node::{EPOCH_LENGTH, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR, TICKET_SUBMISSION_ENDS};
use jam_types::*;
use misc::{bad_order, has_duplicates};
use std::collections::HashSet;
use std::{thread, {sync::{mpsc, Mutex, LazyLock}}};
use tools::log;

pub fn process(
    tickets_extrinsic: &[Ticket],
    safrole_state: &mut Safrole,
    entropy_state: &EntropyPool,
    post_tau: &TimeSlot,
    verifier: &Verifier,
) -> Result<(), ImportError> {

    if tickets_extrinsic.is_empty() {
        return Ok(());
    }

    // Towards the end of the epoch, ticket submission ends implying successive blocks within the same epoch
    // must have an empty tickets extrinsic
    if (*post_tau % EPOCH_LENGTH as TimeSlot) >= TICKET_SUBMISSION_ENDS as TimeSlot {
        log::error!("Unexpected ticket. Block slot: {:?}", post_tau);
        return Err(ImportError::SafroleError(SafroleErrorCode::UnexpectedTicket));
    }

    if tickets_extrinsic.len() > MAX_TICKETS_PER_EXTRINSIC {
        log::error!("Too many tickets: {:?}", tickets_extrinsic.len());
        return Err(ImportError::SafroleError(SafroleErrorCode::TooManyTickets));
    }

    // We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index (a
    // natural number less than TICKET_ENTRIES_PER_VALIDATOR) and a proof of ticket validity.
    for i in 0..tickets_extrinsic.len() {
        if tickets_extrinsic[i].attempt >= TICKET_ENTRIES_PER_VALIDATOR {
            log::error!("Bad ticket attempt: {:?}", tickets_extrinsic[i].attempt);
            return Err(ImportError::SafroleError(SafroleErrorCode::BadTicketAttempt));
        }
    }

    let fixed_input_data: &[u8] = &[&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode()].concat();
    
    // Verify each ticket
    let (tx, rx) = mpsc::channel();

    thread::scope(|s| {
        for (i, ticket) in tickets_extrinsic.iter().enumerate() {
            let tx = tx.clone();
            s.spawn(move || {
                let r = ticket_seal_verify(verifier, ticket, fixed_input_data);
                let _ = tx.send((i, r));
            });
        }
    });

    // Empty the tx channel
    drop(tx);

    let mut enum_result = Vec::new();
    for (i, r) in rx {
        match r {
            Ok(ticket) => enum_result.push((i, ticket)),
            Err(e) => return Err(e),
        }
    }

    // Sort again the tickets after the verification
    enum_result.sort_by_key(|(index, _)| *index);
    // Collect the ticket bodies
    let result: Vec<TicketBody> = enum_result.iter().map(|(_, ticket_body)| ticket_body.clone() ).collect();
    // Collect the tickets ids
    let new_ticket_ids: Vec<OpaqueHash> = result.iter().map(|ticket| ticket.id).collect();
    // Update the ticket accumulator
    safrole_state.ticket_accumulator.extend(result);

    // Check tickets order
    if bad_order(&new_ticket_ids) {
        log::error!("Bad tickets order");
        return Err(ImportError::SafroleError(SafroleErrorCode::BadTicketOrder));
    }

    // Check if there are duplicate tickets
    let ids: Vec<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|ticket| ticket.id.clone()).collect();
    if has_duplicates(&ids) {
        log::error!("Duplicate ticket");
        return Err(ImportError::SafroleError(SafroleErrorCode::DuplicateTicket));
    }

    // Sort tickets
    safrole_state.ticket_accumulator.sort();

    // Remove tickets with low score to make space for new ones
    if safrole_state.ticket_accumulator.len() > EPOCH_LENGTH {
        safrole_state.ticket_accumulator.drain(EPOCH_LENGTH..);
    }

    //  It is invalid to include useless tickets in extrinsic, so all submitted tickets must exist in their posterior ticket accumulator
    let surviving_ids: std::collections::HashSet<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|t| t.id).collect();
    for id in &new_ticket_ids {
        if !surviving_ids.contains(id) {
            log::error!("Ticket {} dropped: submitted ticket did not survive accumulator truncation", tools::hex::encode(id));
            return Err(ImportError::SafroleError(SafroleErrorCode::TicketDropped));
        }
    }

    log::debug!("Ticket accumulator len={:?}", safrole_state.ticket_accumulator.len());

    Ok(())
}

fn ticket_seal_verify(verifier: &Verifier, ticket: &Ticket, fixed_input_data: &[u8]) -> Result<TicketBody, ImportError> {

    let vrf_input_data = [fixed_input_data, &encode_unsigned(ticket.attempt as usize)].concat();
    let aux_data = vec![];
    // Verify ticket validity
    match verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &ticket.signature) {
        Ok(result) => {
            return Ok(TicketBody { id: result, attempt: ticket.attempt });
        },
        Err(_) => { 
            log::error!("Bad ticket proof. Ticket signature: {}", tools::print_hash!(ticket.signature)); 
            return Err(ImportError::SafroleError(SafroleErrorCode::BadTicketProof)); 
        }
    }
}

static TICKET_POOL: LazyLock<Mutex<Vec<Ticket>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn clear_pool() {
    TICKET_POOL.lock().unwrap().clear();
}

pub fn store(ticket: Ticket) {
    log::debug!("Storing ticket {} attempt={}", tools::print_hash!(ticket.signature), ticket.attempt);
    TICKET_POOL.lock().unwrap().push(ticket);
}

pub fn select_for_block_inclusion(
    slot: TimeSlot, 
    verifier: Verifier,
    safrole_state: &Safrole,
    entropy_state: &EntropyPool,
) -> Vec<Ticket> {

    use constants::node::{EPOCH_LENGTH, MAX_TICKETS_PER_EXTRINSIC};

    let existing_ids: Vec<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|t| t.id).collect();

    let fixed_input_data: Vec<u8> = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode()].concat();

    let mut pool = TICKET_POOL.lock().unwrap();

    let mut valid_tickets: Vec<(Ticket, OpaqueHash)> = Vec::new();

    pool.retain(|ticket| {
        let vrf_input = [&fixed_input_data[..], &ticket.attempt.encode()].concat();
        match verifier.ring_vrf_verify(&vrf_input, &[], &ticket.signature) {
            Ok(id) => {
                if existing_ids.contains(&id) {
                    log::debug!("Ticket already in accumulator, discarding");
                    false
                } else {
                    valid_tickets.push((ticket.clone(), id));
                    false
                }
            }
            Err(_) => {
                log::error!("Invalid ticket proof, discarding");
                false
            }
        }
    });

    valid_tickets.sort_by(|a, b| a.1.cmp(&b.1));
    let candidates: Vec<(Ticket, OpaqueHash)> = valid_tickets.into_iter()
        .take(MAX_TICKETS_PER_EXTRINSIC)
        .collect();

    // Simulate insertion into the accumulator to discard tickets that would be dropped
    let candidate_ids: Vec<OpaqueHash> = candidates.iter().map(|(_, id)| *id).collect();
    let mut simulated_acc: Vec<OpaqueHash> = existing_ids;
    simulated_acc.extend(&candidate_ids);
    simulated_acc.sort();

    let selected: Vec<Ticket> = if simulated_acc.len() > EPOCH_LENGTH {
        let survivors: HashSet<OpaqueHash> = simulated_acc[..EPOCH_LENGTH].iter().cloned().collect();
        candidates.into_iter()
            .filter(|(_, id)| survivors.contains(id))
            .map(|(ticket, _)| ticket)
            .collect()
    } else {
        candidates.into_iter()
            .map(|(ticket, _)| ticket)
            .collect()
    };

    log::debug!("Selected {} tickets for block at slot {}", selected.len(), slot);

    selected
}

pub fn generate(
    bandersnatch_secret_key: BandersnatchSecret,
    bandersnatch_public: BandersnatchPublic,
) -> Vec<(Ticket, OpaqueHash)> {

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
        log::info!("Generated ticket attempt={} id={}", attempt, tools::hex::encode(&ticket_id));
    }

    tickets
}