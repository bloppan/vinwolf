extern crate hex;
extern crate array_bytes;

use crate::types::{BandersnatchKey, Ed25519Key, BlsKey, BandersnatchRingCommitment, Metadata, OpaqueHash, TimeSlot};
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};

use crate::codec::{Encode, EncodeSize};
use crate::codec::safrole::{Input as Input, Output, SafroleState, KeySet, Safrole,
                            ErrorType, EpochMark, OutputMarks, ValidatorData, 
                            TicketsOrKeys, TicketBody};

mod bandersnatch;

use serde::Deserialize;
use sp_core::blake2_256;

// Update Safrole state
pub fn update_state(input: Input, state: &mut SafroleState) -> Output {

    if input.slot <= state.tau {
        return Output::err(ErrorType::bad_slot);
    }

    if input.extrinsic.len() > 0 {
        if input.slot >= TICKET_SUBMISSION_ENDS as u32 {
            return Output::err(ErrorType::unexpected_ticket);
        }

        let validity = bandersnatch::verify_tickets(&input, state);
        
        if let Output::err(error_type) = validity {
            return Output::err(error_type);
        }
    }
    // Calculate time parameters
    let e: u32 = state.tau / EPOCH_LENGTH as u32;
    let m: u32 = state.tau % EPOCH_LENGTH as u32;
    let post_e: u32 = input.slot / EPOCH_LENGTH as u32;
    let post_m: u32 = input.slot % EPOCH_LENGTH as u32;
    
    // Output marks
    let mut epoch_mark: Option<EpochMark> = None;
    let mut tickets_mark: Option<Vec<TicketBody>> = None;

    // Check if we are in a new epoch (e' > e)
    if post_e > e {
        update_entropy_pool(state); 
        key_rotation(&input, state);
        epoch_mark = Some(EpochMark {
            entropy: state.eta[1].clone(),
            validators: state.gamma_k
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect::<Vec<BandersnatchKey>>()  
                .try_into()  
                .expect("Incorrect number of validators"),  
        });
        if post_e == e + 1 && m >= TICKET_SUBMISSION_ENDS as u32 && state.gamma_a.len() == EPOCH_LENGTH {
            state.gamma_s = TicketsOrKeys::tickets(outside_in_sequencer(&state.gamma_a));
        } else if post_e == e {
            // gamma_s' = gamma_s
        } else {
            let bandersnatch_keys: Vec<_> = state.kappa
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect();
            state.gamma_s = TicketsOrKeys::keys(fallback(&state.eta[2], &bandersnatch_keys));
        } 
        state.gamma_a = vec![];
    } else if post_e == e && m < TICKET_SUBMISSION_ENDS as u32 && TICKET_SUBMISSION_ENDS  as u32 <= post_m && state.gamma_a.len() == EPOCH_LENGTH {
        tickets_mark = Some(outside_in_sequencer(&state.gamma_a));
    }
    state.tau = input.slot; // tau' = slot

    // Update recent entropy eta[0]
    update_recent_entropy(&input, state);
    return Output::ok(OutputMarks {epoch_mark, tickets_mark});
}

// Update the three aditional accumulator's values (Eq 68)
fn update_entropy_pool(state: &mut SafroleState) {
    state.eta[3] = state.eta[2].clone();
    state.eta[2] = state.eta[1].clone();
    state.eta[1] = state.eta[0].clone();
}

// update eta'[0] (Equation 67)
fn update_recent_entropy(input: &Input, state: &mut SafroleState) {
    state.eta[0] = blake2_256(&[state.eta[0], input.entropy].concat());
}

fn set_offenders_null(input: &Input, state: &SafroleState) -> Box<[ValidatorData; VALIDATORS_COUNT]> {

    if input.post_offenders.is_empty() {
        return Box::new(*state.iota.clone());
    }

    let mut iota = state.iota.clone();

    for offender in &input.post_offenders {
        for key in &mut *iota {
            if *offender == key.ed25519 {
                key.bandersnatch = [0u8; std::mem::size_of::<BandersnatchKey>()];
                key.ed25519 = [0u8; std::mem::size_of::<Ed25519Key>()];
                key.bls = [0u8; std::mem::size_of::<BlsKey>()];
                key.metadata = [0u8; std::mem::size_of::<Metadata>()];
            }
        }
    }
    return Box::new(*iota);
}

// Equation 58
fn key_rotation(input: &Input, state: &mut SafroleState) { 
    state.lambda = state.kappa.clone();
    state.kappa = state.gamma_k.clone();
    state.gamma_k = Box::new(*set_offenders_null(&input, &state));
    let bandersnatch_keys = state.gamma_k
                                    .iter()
                                    .map(|validator| validator.bandersnatch.clone())
                                    .collect();
    state.gamma_z = bandersnatch::create_root_epoch(&bandersnatch_keys);
}

//Equation 70
fn outside_in_sequencer(tickets: &Vec<TicketBody>) -> Vec<TicketBody> {
    let mut new_ticket_accumulator: Vec<TicketBody> = Vec::with_capacity(tickets.len());
    let mut i = 0;
    let n_seq = tickets.len() / 2; 

    while i < n_seq {
        new_ticket_accumulator.push(tickets[i].clone());
        new_ticket_accumulator.push(tickets[tickets.len() - 1 - i].clone());
        i += 1;
    }
    if tickets.len() % 2 != 0 {
        new_ticket_accumulator.push(tickets[n_seq].clone());
    }
    new_ticket_accumulator
}

//Equation 71
fn fallback(entropy: &OpaqueHash, keys: &Vec<BandersnatchKey>) -> Box<[BandersnatchKey; EPOCH_LENGTH]> {

    let mut new_keys: Box<[BandersnatchKey; EPOCH_LENGTH]> = Box::new([[0u8; std::mem::size_of::<OpaqueHash>()]; EPOCH_LENGTH]);

    for i in 0u32..EPOCH_LENGTH as u32 { 
        let index_le = i.encode();
        let hash = blake2_256(&[&entropy[..], &index_le].concat());
        let hash_4 = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id = (hash_4 % VALIDATORS_COUNT as u32) as usize;
        new_keys[i as usize] = keys[id].clone();
    }
    new_keys
}
