extern crate hex;
extern crate array_bytes;

use crate::types::{BandersnatchKey, Ed25519Key, BlsKey, BandersnatchRingCommitment, Metadata, OpaqueHash, TimeSlot};
use crate::globals::{NUM_VALIDATORS, EPOCH_LENGTH};

//pub mod codec;
use crate::codec::{Encode};
use crate::codec::safrole::{Input as Input, Output, SafroleState, KeySet, Safrole,
                            ErrorType, EpochMark, OutputMarks, ValidatorData, 
                            TicketsOrKeys, TicketBody};

mod bandersnatch;
mod time;


use serde::Deserialize;
use sp_core::blake2_256;

// The length of an epoch in timeslots
pub const E: u32 = 12; // The length of an epoch timeslots.
const Y: u32 = 10; // The number of slots into an epoch at which ticket-submission ends
const V: u32 = 6;  // Total number of validators

// Update Safrole state
pub fn update_state(input: Input, state: &mut SafroleState) -> Output {

    if input.slot <= state.tau {
        return Output::err(ErrorType::bad_slot);
    }

    if input.extrinsic.len() > 0 {
        if input.slot >= Y {
            return Output::err(ErrorType::unexpected_ticket);
        }

        let validity = bandersnatch::verify_tickets(input.clone(), state);
        
        if let Output::err(error_type) = validity {
            return Output::err(error_type);
        }
    }
    // Calculate time parameters
    let e: u32 = state.tau / E;
    let m: u32 = state.tau % E;
    let post_e: u32 = input.slot / E;
    let post_m: u32 = input.slot % E;
    
    // Output marks
    let mut epoch_mark: Option<EpochMark> = None;
    let mut tickets_mark: Option<Vec<TicketBody>> = None;
    // Check if we are in a new epoch (e' > e)
    if post_e > e {
        update_entropy_pool(state); 
        key_rotation(input.clone(), state);
        epoch_mark = Some(EpochMark {
            entropy: state.eta[1].clone(),
            validators: state.gamma_k
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect::<Vec<BandersnatchKey>>()  // Primero recolectamos en un Vec
                .try_into()  // Luego intentamos convertirlo a un array fijo
                .expect("Incorrect number of validators"),  // Asegúrate de que el número sea correcto
        });
        if post_e == e + 1 && m >= Y && state.gamma_a.len() == E as usize {
            state.gamma_s = TicketsOrKeys::tickets(outside_in_sequencer(state.gamma_a.clone()));
        } else if post_e == e {
            // gamma_s' = gamma_s
        } else {
            let bandersnatch_keys: Vec<_> = state.kappa
            .iter()
            .map(|validator| validator.bandersnatch.clone())
            .collect();// bandersnatch_keys_collect(state.clone(), KeySet::kappa);
            state.gamma_s = TicketsOrKeys::keys(Box::new(fallback(state.eta[2].clone(), bandersnatch_keys.clone())));
        } 
        state.gamma_a = vec![];
    } else if post_e == e && m < Y && Y <= post_m && state.gamma_a.len() == E as usize {
        println!("----------------------------------------------------------");
        tickets_mark = Some(outside_in_sequencer(state.gamma_a.clone()));
    }
    state.tau = input.slot; // tau' = slot
    // Update recent entropy eta[0]
    update_recent_entropy(input.clone(), state);
    return Output::ok(OutputMarks {epoch_mark, tickets_mark});
}

// Update the three aditional accumulator's values (Eq 68)
fn update_entropy_pool(state: &mut SafroleState) {
    let eta_0 = state.eta[0].clone();
    let eta_1 = state.eta[1].clone();
    let eta_2 = state.eta[2].clone();
    state.eta[1] = eta_0.clone();
    state.eta[2] = eta_1.clone();
    state.eta[3] = eta_2.clone();
}

// update eta'[0] (Equation 67)
fn update_recent_entropy(input: Input, state: &mut SafroleState) {
    
    /*let clean_eta0 = &state.eta[0][2..];
    let clean_entropy = &input.entropy[2..];
    let eta0_bytes = array_bytes::hex2bytes(clean_eta0).expect("Failed to convert hex to bytes");
    let entropy_bytes = array_bytes::hex2bytes(clean_entropy).expect("Failed to convert hex to bytes");*/
    //let concatenated = [state.eta[0], input.entropy].concat();
    //let hash = blake2_256(&concatenated);
    state.eta[0] = blake2_256(&[state.eta[0], input.entropy].concat());
}

/*pub fn bandersnatch_keys_collect(state: SafroleState, key_set: KeySet) -> [BandersnatchKey; NUM_VALIDATORS] {
    let bandersnatch_keys = match key_set {
        KeySet::gamma_k => state.gamma_k.bandersnatch.clone(),
        KeySet::kappa => state.kappa.bandersnatch.clone(),
    };
    bandersnatch_keys
}*/

fn set_offenders_null(input: &Input, state: &SafroleState) -> [ValidatorData; NUM_VALIDATORS] {

    if input.post_offenders.is_empty() {
        return *state.iota.clone();
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
    return *iota;
}

// Equation 58
fn key_rotation(input: Input, state: &mut SafroleState) { 
    // bandersnatch_keys_collect(state.clone(), KeySet::gamma_k);
    state.lambda = state.kappa.clone();
    state.kappa = state.gamma_k.clone();
    state.gamma_k = Box::new(set_offenders_null(&input, &state));
    let bandersnatch_keys = state.gamma_k
    .iter()
    .map(|validator| validator.bandersnatch.clone())
    .collect();
    state.gamma_z = bandersnatch::create_root_epoch(bandersnatch_keys);
}

//Equation 70
fn outside_in_sequencer(tickets: Vec<TicketBody>) -> Vec<TicketBody> {
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
fn fallback(entropy: OpaqueHash, keys: Vec<BandersnatchKey>) -> [BandersnatchKey; E as usize] {
    let mut new_keys: [BandersnatchKey; E as usize] = [[0u8; std::mem::size_of::<OpaqueHash>()]; E as usize];
    let clean_entropy = entropy;

    for i in 0u32..E as u32 { 
        let index_le = i.encode();
        /*let index_hex = hex::encode(index_le);
        let entropy_bytes = array_bytes::hex2bytes(clean_entropy).expect("Failed to convert hex to bytes");
        let index_bytes = array_bytes::hex2bytes(index_hex).expect("Failed to convert hex to bytes");
        let concatenated = [entropy_bytes, index_bytes].concat();*/
        let concatenated = [&entropy[..], &index_le[..]].concat();
        let hash = blake2_256(&concatenated);
        let hash_4 = u32::from_be_bytes([hash[3], hash[2], hash[1], hash[0]]);
        let id = (hash_4 % V as u32) as usize;
        new_keys[i as usize] = keys[id].clone();
    }
    new_keys
}
