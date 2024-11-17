/// Jam's block production mechanism, termed Safrole after the novel Sassafras production mechanism of which it is 
/// a simplified variant, is a stateful system rather more complex than the Nakamoto consensus described in the YP.
///
/// The chief purpose of a block production consensus mechanism is to limit the rate at which new blocks may be
/// authored and, ideally, preclude the possibility of "forks": multiple blocks with equal numbers of ancestors.
///
/// To achieve this, Safrole limits the possible author of any block within any given six-second timeslot to a single 
/// key-holder from within a prespecified set of validators. Furthermore, under normal operation, the identity of the
/// key-holder of any future timeslot will have a very high degree of anonymity. As a side effect of its operation, we
/// can generate a high-quality pool of entropy which may be used by other parts of the protocol and is accessible to
/// services running on it. 
///
/// Because of its tightly scoped role, the core of Safrole's state, "gamma", is independent of the 
/// rest of the protocol. It interacts with other portions of the protocol through "iota" and "kappa", the prospective 
/// and active sets of validator keys respectively; "tau" , the most recent block's timeslot; and "eta", the entropy 
/// accumulator.
///
/// The Safrole protocol generates, once per epoch, a sequence of "EPOCH_LENGTH" sealing keys, one for each potential 
/// block within a whole epoch. Each block header includes its timeslot index Ht (the number of six-second periods since the 
/// Jam Common Era began) and a valid seal signature Hs, signed by the sealing key corresponding to the timeslot within 
/// the aforementioned sequence. Each sealing key is in fact a pseudonym for some validator which was agreed the privilege 
/// of authoring a block in the corresponding timeslot.
///
/// In order to generate this sequence of sealing keys in regular operation, and in particular to do so without making 
/// public the correspondence relation between them and the validator set, we use a novel cryptographic structure known as 
/// a Ringvrf, utilizing the Bandersnatch curve. Bandersnatch Ringvrf allows for a proof to be provided which simultaneously 
/// guarantees the author controlled a key within a set (in our case validators), and secondly provides an output, an 
/// unbiasable deterministic hash giving us a secure verifiable random function (vrf). This anonymous and secure random 
/// output is a ticket and validators' tickets with the best score define the new sealing keys allowing the chosen validators 
/// to exercise their privilege and create a new block at the appropriate time.

use crate::types::{BandersnatchKey, Ed25519Key, BlsKey, Metadata, OpaqueHash};
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use crate::codec::{Encode};
use crate::codec::safrole::{Input, Output, SafroleState, ErrorType, OutputMarks, ValidatorData, TicketsOrKeys};
use crate::codec::header::{EpochMark, TicketBody};

use sp_core::blake2_256;
mod bandersnatch;

// Update Safrole state
pub fn update_state(input: Input, state: &mut SafroleState) -> Output {
    // tau defines de most recent block
    // Timeslot must be strictly monotonic
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
        // On an epoch transition, we therefore rotate the accumulator value into the history eta1, eta2 eta3
        rotate_entropy_pool(state); 
        // With a new epoch, validator keys get rotated and the epoch's Bandersnatch key root is updated
        key_rotation(&input, state);
        // If the block is the first in a new epoch, then a tuple of the epoch randomness and a sequence of 
        // Bandersnatch keys defining the Bandersnatch validator keys beginning in the next epoch
        epoch_mark = Some(EpochMark {
            entropy: state.eta[1].clone(),
            validators: state.gamma_k
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect::<Vec<BandersnatchKey>>()  
                .try_into()  
                .expect("Incorrect number of validators"),  
        });
        // gamma_s is the current epoch's slot-sealer series, which is either a full complement of EPOCH_LENGTH tickets
        // or, in case of fallback, a series of EPOCH_LENGTH bandersnatch keys
        if post_e == e + 1 && m >= TICKET_SUBMISSION_ENDS as u32 && state.gamma_a.len() == EPOCH_LENGTH {
            // If the block is the first after the end of the submission period for tickets and if the ticket accumulator 
            // is saturated, then the final sequence of ticket identifiers
            state.gamma_s = TicketsOrKeys::Tickets(outside_in_sequencer(&state.gamma_a));
        } else if post_e == e {
            // gamma_s' = gamma_s
        } else {
            // Otherwise, it takes the value of the fallback key sequence
            let bandersnatch_keys: Vec<_> = state.kappa
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect();
            state.gamma_s = TicketsOrKeys::Keys(fallback(&state.eta[2], &bandersnatch_keys));
        } 
        state.gamma_a = vec![];
    } else if post_e == e && m < TICKET_SUBMISSION_ENDS as u32 && TICKET_SUBMISSION_ENDS  as u32 <= post_m && state.gamma_a.len() == EPOCH_LENGTH {
        // gamma_a is the ticket accumulator, a series of highestscoring ticket identifiers to be used for the next epoch.
        tickets_mark = Some(outside_in_sequencer(&state.gamma_a));
    }
    // tau defines the most recent block's index
    state.tau = input.slot;

    // Update recent entropy eta0
    update_recent_entropy(&input, state);
    return Output::ok(OutputMarks {epoch_mark, tickets_mark});
}

fn update_recent_entropy(input: &Input, state: &mut SafroleState) {
    // eta0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over 
    // some unbiasable input, is combined each block. eta1 and eta2 meanwhile retain the state of this accumulator at the end 
    // of the two most recently ended epochs in order.
    state.eta[0] = blake2_256(&[state.eta[0], input.entropy].concat());
}

fn rotate_entropy_pool(state: &mut SafroleState) {
    // In addition to the entropy accumulator eta0, we retain three additional historical values of the accumulator at the point of 
    // each of the three most recently ended epochs, eta1, eta2 and eta3. The second-oldest of these eta2 is utilized to help ensure 
    // future entropy is unbiased and seed the fallback seal-key generation function with randomness. The oldest is used to regenerate 
    // this randomness when verifying the seal gamma_s.
    state.eta[3] = state.eta[2].clone();
    state.eta[2] = state.eta[1].clone();
    state.eta[1] = state.eta[0].clone();
}

fn set_offenders_null(input: &Input, state: &SafroleState) -> Vec<ValidatorData> {
    // We return the same iota keyset if there aren't offenders
    if input.post_offenders.is_empty() {
        return state.iota.clone();
    }
    let mut iota = state.iota.clone();
    // For each offender set ValidatorData to zero
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
    return iota;
}

fn key_rotation(input: &Input, state: &mut SafroleState) { 
    // In addition to the active set of validator keys "kappa" and staging set "iota", internal to the Safrole state 
    // we retain a pending set "gamma_k".
    state.lambda = state.kappa.clone();
    state.kappa = state.gamma_k.clone();
    // In addition to the active set of validator keys "kappa" and staging set "iota", internal to the Safrole state 
    // we retain a pending set "gamma_k". The active set is the set of keys identifying the nodes which are 
    // currently privileged to author blocks and carry out the validation processes, whereas the pending set 
    // "gamma_k", which is reset to "iota" at the beginning of each epoch, is the set of keys which will be 
    // active in the next epoch and which determine the Bandersnatch ring root which authorizes tickets into 
    // the sealing-key contest for the next epoch.
    //
    // The posterior queued validator key set "gamma_k" is defined such that incoming keys belonging to the offenders 
    // are replaced with a null key containing only zeroes.
    state.gamma_k = set_offenders_null(&input, &state);
    let bandersnatch_keys = state.gamma_k
                                    .iter()
                                    .map(|validator| validator.bandersnatch.clone())
                                    .collect();
    // With a new epoch under regular conditions, validator keys get rotated and the epoch's Bandersnatch key root is
    // updated into "gamma_z"
    state.gamma_z = bandersnatch::create_root_epoch(&bandersnatch_keys);
}

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

fn fallback(entropy: &OpaqueHash, keys: &Vec<BandersnatchKey>) -> Vec<BandersnatchKey> {
    // This is the fallback key sequence function which selects an epoch's worth of validator Bandersnatch keys from the 
    // validator key set using the entropy collected on-chain
    let mut new_keys: Vec<BandersnatchKey> = Vec::with_capacity(std::mem::size_of::<BandersnatchKey>() * EPOCH_LENGTH);
    for i in 0u32..EPOCH_LENGTH as u32 { 
        let index_le = i.encode();
        let hash = blake2_256(&[&entropy[..], &index_le].concat());
        let hash_4 = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id = (hash_4 % VALIDATORS_COUNT as u32) as usize;
        new_keys.push(keys[id].clone());
    }
    new_keys
}
