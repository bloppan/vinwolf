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

use sp_core::blake2_256;

use crate::types::{
    BandersnatchPublic, BandersnatchRingCommitment, BlsPublic, Ed25519Public, EntropyPool, EpochMark, Metadata, OpaqueHash, 
    OutputDataSafrole, OutputSafrole, Safrole, SafroleErrorCode, TicketBody, TicketsExtrinsic, TicketsOrKeys, TimeSlot, 
    ValidatorData, ValidatorsData, Entropy
};
use crate::constants::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use crate::blockchain::state::ProcessError;
use crate::blockchain::state::entropy::rotate_entropy_pool;
use crate::utils::common::set_offenders_null;
use crate::utils::codec::Encode;

mod bandersnatch;

impl Default for Safrole {
    fn default() -> Self {
        Safrole {
            pending_validators: ValidatorsData::default(),
            ticket_accumulator: vec![TicketBody::default()],
            seal: TicketsOrKeys::None,
            epoch_root: [0u8; std::mem::size_of::<BandersnatchRingCommitment>()],
        }
    }
}

// Process Safrole state
pub fn process_safrole(
    safrole_state: &mut Safrole,
    entropy_state: &mut EntropyPool,
    curr_validators: &mut ValidatorsData,
    prev_validators: &mut ValidatorsData,
    tau: &mut TimeSlot,
    slot: &TimeSlot,
    entropy: &Entropy,
    tickets_extrinsic: &TicketsExtrinsic,
    post_offenders: &Vec<Ed25519Public>,
) -> Result<OutputDataSafrole, ProcessError> {

    // tau defines de most recent block
    // Timeslot must be strictly monotonic

    if *slot <= *tau {
        return Err(ProcessError::SafroleError(SafroleErrorCode::BadSlot));
    }

    if tickets_extrinsic.len() > 0 {
        if *slot >= TICKET_SUBMISSION_ENDS as TimeSlot {
            return Err(ProcessError::SafroleError(SafroleErrorCode::UnexpectedTicket));
        }

        let validity = bandersnatch::verify_tickets(safrole_state, entropy_state, tickets_extrinsic);
        
        if let Err(error_type) = validity {
            return Err(error_type);
        }
    }
    // Calculate time parameters
    let epoch = *tau / EPOCH_LENGTH as TimeSlot;
    let m = *tau % EPOCH_LENGTH as TimeSlot;
    let post_epoch= *slot / EPOCH_LENGTH as TimeSlot;
    let post_m = *slot % EPOCH_LENGTH as TimeSlot;
    
    // Output marks
    let mut epoch_mark: Option<EpochMark> = None;
    let mut tickets_mark: Option<Vec<TicketBody>> = None;

    // Check if we are in a new epoch (e' > e)
    if post_epoch > epoch {
        // On an epoch transition, we therefore rotate the accumulator value into the history eta1, eta2 eta3
        rotate_entropy_pool(entropy_state); 
        // With a new epoch, validator keys get rotated and the epoch's Bandersnatch key root is updated
        key_rotation(safrole_state, curr_validators, prev_validators, post_offenders);
        // If the block is the first in a new epoch, then a tuple of the epoch randomness and a sequence of 
        // Bandersnatch keys defining the Bandersnatch validator keys beginning in the next epoch
        epoch_mark = Some(EpochMark {
            entropy: entropy_state.0[1].clone(),
            tickets_entropy: entropy_state.0[2].clone(),
            validators: safrole_state.pending_validators.0
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect::<Vec<BandersnatchPublic>>()  
                .try_into()  
                .expect("Incorrect number of validators"),  
        });
        // gamma_s is the current epoch's slot-sealer series, which is either a full complement of EPOCH_LENGTH tickets
        // or, in case of fallback, a series of EPOCH_LENGTH bandersnatch keys
        if post_epoch == epoch + 1 && m >= TICKET_SUBMISSION_ENDS as u32 && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
            // If the block is the first after the end of the submission period for tickets and if the ticket accumulator 
            // is saturated, then the final sequence of ticket identifiers
            safrole_state.seal = TicketsOrKeys::Tickets(outside_in_sequencer(&safrole_state.ticket_accumulator));
        } else if post_epoch == epoch {
            // gamma_s' = gamma_s
        } else {
            // Otherwise, it takes the value of the fallback key sequence
            let bandersnatch_keys: Vec<_> = curr_validators.0
                .iter()
                .map(|validator| validator.bandersnatch.clone())
                .collect();
            safrole_state.seal = TicketsOrKeys::Keys(fallback(&entropy_state.0[2], &bandersnatch_keys));
        } 
        safrole_state.ticket_accumulator = vec![];
    } else if post_epoch == epoch && m < TICKET_SUBMISSION_ENDS as u32 && TICKET_SUBMISSION_ENDS  as u32 <= post_m && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
        // gamma_a is the ticket accumulator, a series of highestscoring ticket identifiers to be used for the next epoch.
        tickets_mark = Some(outside_in_sequencer(&safrole_state.ticket_accumulator));
    }
    // tau defines the most recent block's index
    *tau = *slot;

    // Update recent entropy eta0
    update_recent_entropy(entropy_state, entropy.clone());
    
    return Ok(OutputDataSafrole {epoch_mark, tickets_mark});
}

fn update_recent_entropy(entropy_state: &mut EntropyPool, new_entropy: Entropy) {
    // eta0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over 
    // some unbiasable input, is combined each block. eta1 and eta2 meanwhile retain the state of this accumulator at the end 
    // of the two most recently ended epochs in order.
    entropy_state.0[0] = Entropy(blake2_256(&[entropy_state.0[0].0, new_entropy.0].concat()));
}

fn key_rotation(
    safrole_state: &mut Safrole, 
    curr_validators: &mut ValidatorsData, 
    prev_validators: &mut ValidatorsData,
    post_offenders: &Vec<Ed25519Public>
) { 
    // In addition to the active set of validator keys "kappa" and staging set "iota", internal to the Safrole state 
    // we retain a pending set "gamma_k".
    *prev_validators = curr_validators.clone();
    *curr_validators = safrole_state.pending_validators.clone();    
    // In addition to the active set of validator keys "kappa" and staging set "iota", internal to the Safrole state 
    // we retain a pending set "gamma_k". The active set is the set of keys identifying the nodes which are 
    // currently privileged to author blocks and carry out the validation processes, whereas the pending set 
    // "gamma_k", which is reset to "iota" at the beginning of each epoch, is the set of keys which will be 
    // active in the next epoch and which determine the Bandersnatch ring root which authorizes tickets into 
    // the sealing-key contest for the next epoch.
    //
    // The posterior queued validator key set "gamma_k" is defined such that incoming keys belonging to the offenders 
    // are replaced with a null key containing only zeroes.
    set_offenders_null(&mut safrole_state.pending_validators, &post_offenders);

    let bandersnatch_keys = safrole_state.pending_validators.0
                                                                            .iter()
                                                                            .map(|validator| validator.bandersnatch.clone())
                                                                            .collect();
    // With a new epoch under regular conditions, validator keys get rotated and the epoch's Bandersnatch key root is
    // updated into "gamma_z"
    safrole_state.epoch_root = bandersnatch::create_root_epoch(&bandersnatch_keys);
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

fn fallback(entropy: &Entropy, keys: &Vec<BandersnatchPublic>) -> Vec<BandersnatchPublic> {
    // This is the fallback key sequence function which selects an epoch's worth of validator Bandersnatch keys from the 
    // validator key set using the entropy collected on-chain
    let mut new_keys: Vec<BandersnatchPublic> = Vec::with_capacity(std::mem::size_of::<BandersnatchPublic>() * EPOCH_LENGTH);
    for i in 0u32..EPOCH_LENGTH as u32 { 
        let index_le = i.encode();
        let hash = blake2_256(&[&entropy.0[..], &index_le].concat());
        let hash_4 = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id = (hash_4 % VALIDATORS_COUNT as u32) as usize;
        new_keys.push(keys[id].clone());
    }
    new_keys
}
