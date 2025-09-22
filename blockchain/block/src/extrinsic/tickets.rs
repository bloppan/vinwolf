/*
    Tickets Extrinsic is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal “contest” to determine 
    which validators are privileged to author a block for each timeslot in the following epoch. Tickets specify an entry index 
    together with a proof of ticket’s validity. The proof implies a ticket identifier, a high-entropy unbiasable 32-octet sequence, 
    which is used both as a score in the aforementioned contest and as input to the on-chain vrf. Towards the end of the epoch 
    (i.e. Y slots from the start) this contest is closed implying successive blocks within the same epoch must have an empty tickets 
    extrinsic. At this point, the following epoch’s seal key sequence becomes fixed. We define the extrinsic as a sequence of proofs 
    of valid tickets, each of which is a tuple of an entry index (a natural number less than N) and a proof of ticket validity.
*/

use utils::bandersnatch::Verifier;
use std::thread;
use std::sync::mpsc;
use constants::node::{EPOCH_LENGTH, TICKET_SUBMISSION_ENDS, MAX_TICKETS_PER_EXTRINSIC, TICKET_ENTRIES_PER_VALIDATOR};
use jam_types::{EntropyPool, OpaqueHash, Safrole, SafroleErrorCode, TicketBody, TimeSlot, Ticket, ProcessError};
use codec::Encode;
use utils::{common::{has_duplicates, bad_order}, log};

pub fn process(
    tickets_extrinsic: &[Ticket],
    safrole_state: &mut Safrole,
    entropy_state: &EntropyPool,
    post_tau: &TimeSlot,
    verifier: &Verifier,
) -> Result<(), ProcessError> {

    if tickets_extrinsic.is_empty() {
        return Ok(());
    }

    // Towards the end of the epoch, ticket submission ends implying successive blocks within the same epoch
    // must have an empty tickets extrinsic
    if (*post_tau % EPOCH_LENGTH as TimeSlot) >= TICKET_SUBMISSION_ENDS as TimeSlot {
        log::error!("Unexpected ticket. Block slot: {:?}", post_tau);
        return Err(ProcessError::SafroleError(SafroleErrorCode::UnexpectedTicket));
    }

    if tickets_extrinsic.len() > MAX_TICKETS_PER_EXTRINSIC {
        log::error!("Too many tickets: {:?}", tickets_extrinsic.len());
        return Err(ProcessError::SafroleError(SafroleErrorCode::TooManyTickets));
    }

    // We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index (a
    // natural number less than TICKET_ENTRIES_PER_VALIDATOR) and a proof of ticket validity.
    for i in 0..tickets_extrinsic.len() {
        if tickets_extrinsic[i].attempt >= TICKET_ENTRIES_PER_VALIDATOR {
            log::error!("Bad ticket attempt: {:?}", tickets_extrinsic[i].attempt);
            return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketAttempt));
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

    /*let verify_result = tickets_extrinsic
            .par_iter()
            .try_fold(
                || Vec::new(),
                |mut tickets_acc: Vec<TicketBody>, ticket| {
                    match ticket_seal_verify(verifier, ticket, &fixed_input_data) {
                        Ok(ticket_body) => {
                            tickets_acc.push(ticket_body);
                            Ok(tickets_acc)
                        }
                        Err(e) => Err(e),
                    }
                },
            )
            .try_reduce(
                || Vec::new(),
                |mut tickets_acc, ticket_body| {
                    tickets_acc.extend(ticket_body);
                    Ok(tickets_acc)
                }
            );
    

    match verify_result {

        Ok(result) => {
            new_ticket_ids = result.iter().map(|ticket| ticket.id).collect();
            safrole_state.ticket_accumulator.extend(result);

        },
        Err(_) => return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketProof)) 
    }*/

    /*for i in 0..tickets_extrinsic.len() {

        let vrf_input_data = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode(), &tickets_extrinsic[i].attempt.encode()].concat();
        let aux_data = vec![];
        // Verify ticket validity
        let res = verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &tickets_extrinsic[i].signature);
        match res {
            Ok(result) => {
                new_ticket_ids.push(result);
                safrole_state.ticket_accumulator.push(TicketBody {
                    id: result,
                    attempt: tickets_extrinsic[i].attempt,
                });
            },
            Err(_) => { 
                log::error!("Bad ticket proof. Ticket: {:?} Signature: {}", i, utils::print_hash!(tickets_extrinsic[i].signature)); 
                return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketProof)); 
            }
        }
    }*/
    // Check tickets order
    let start_order = std::time::Instant::now();
    if bad_order(&new_ticket_ids) {
        log::error!("Bad tickets order");
        return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketOrder));
    }
    let end_order = start_order.elapsed();
    log::info!("Order time: {:?}", end_order);

    let start_duplicates = std::time::Instant::now();
    // Check if there are duplicate tickets
    let ids: Vec<OpaqueHash> = safrole_state.ticket_accumulator.iter().map(|ticket| ticket.id.clone()).collect();
    if has_duplicates(&ids) {
        log::error!("Duplicate ticket");
        return Err(ProcessError::SafroleError(SafroleErrorCode::DuplicateTicket));
    }
    let end_duplicates = start_duplicates.elapsed();
    log::info!("Duplicates time: {:?}", end_duplicates);

    let start_sort = std::time::Instant::now();
    // Sort tickets
    safrole_state.ticket_accumulator.sort();
    let end_sort = start_sort.elapsed();
    log::info!("Sort time: {:?}", end_sort);

    let start_drain = std::time::Instant::now();
    // Remove old tickets to make space for new ones
    if safrole_state.ticket_accumulator.len() > EPOCH_LENGTH {
        safrole_state.ticket_accumulator.drain(EPOCH_LENGTH..);
    }
    let end_drain = start_drain.elapsed();
    log::info!("Drain time: {:?}", end_drain);

    //log::debug!("Extrinsic tickets processed succesfully");

    Ok(())
}

    /*
    
    let verifier = Verifier::new(ring_set);
    let mut new_ticket_ids: Vec<OpaqueHash> = vec![];
    // Verify each ticket
    for i in 0..tickets_extrinsic.len() {

        let vrf_input_data = [&b"jam_ticket_seal"[..], &entropy_state.buf[2].encode(), &tickets_extrinsic[i].attempt.encode()].concat();
        let aux_data = vec![];
        // Verify ticket validity
        let res = verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &tickets_extrinsic[i].signature);
        match res {
            Ok(result) => {
                new_ticket_ids.push(result);
                safrole_state.ticket_accumulator.push(TicketBody {
                    id: result,
                    attempt: tickets_extrinsic[i].attempt,
                });
            },
            Err(_) => { 
                log::error!("Bad ticket proof. Ticket: {:?} Signature: {}", i, utils::print_hash!(tickets_extrinsic[i].signature)); 
                return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketProof)); 
            }
        }
    }*/


fn ticket_seal_verify(verifier: &Verifier, ticket: &Ticket, fixed_input_data: &[u8]) -> Result<TicketBody, ProcessError> {

    let vrf_input_data = [fixed_input_data, &ticket.attempt.encode()].concat();
    let aux_data = vec![];
    // Verify ticket validity
    match verifier.ring_vrf_verify(&vrf_input_data, &aux_data, &ticket.signature) {
        Ok(result) => {
            return Ok(TicketBody { id: result, attempt: ticket.attempt });
            /*new_ticket_ids.push(result);
            safrole_state.ticket_accumulator.push(TicketBody {
                id: result,
                attempt: tickets_extrinsic[i].attempt,
            });*/
        },
        Err(_) => { 
            log::error!("Bad ticket proof. Ticket signature: {}", utils::print_hash!(ticket.signature)); 
            return Err(ProcessError::SafroleError(SafroleErrorCode::BadTicketProof)); 
        }
    }
}
