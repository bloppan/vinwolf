/*
    Jam's block production mechanism, termed Safrole after the novel Sassafras production mechanism of which it is 
    a simplified variant, is a stateful system rather more complex than the Nakamoto consensus described in the YP.
    The chief purpose of a block production consensus mechanism is to limit the rate at which new blocks may be
    authored and, ideally, preclude the possibility of "forks": multiple blocks with equal numbers of ancestors.
    
    To achieve this, Safrole limits the possible author of any block within any given six-second timeslot to a single 
    key-holder from within a prespecified set of validators. Furthermore, under normal operation, the identity of the
    key-holder of any future timeslot will have a very high degree of anonymity. As a side effect of its operation, we
    can generate a high-quality pool of entropy which may be used by other parts of the protocol and is accessible to
    services running on it. 
    
    Because of its tightly scoped role, the core of Safrole's state, "gamma", is independent of the 
    rest of the protocol. It interacts with other portions of the protocol through "iota" and "kappa", the prospective 
    and active sets of validator keys respectively; "tau" , the most recent block's timeslot; and "eta", the entropy 
    accumulator.
    
    The Safrole protocol generates, once per epoch, a sequence of "EPOCH_LENGTH" sealing keys, one for each potential 
    block within a whole epoch. Each block header includes its timeslot index Ht (the number of six-second periods since the 
    Jam Common Era began) and a valid seal signature Hs, signed by the sealing key corresponding to the timeslot within 
    the aforementioned sequence. Each sealing key is in fact a pseudonym for some validator which was agreed the privilege 
    of authoring a block in the corresponding timeslot.
    
    In order to generate this sequence of sealing keys in regular operation, and in particular to do so without making 
    public the correspondence relation between them and the validator set, we use a novel cryptographic structure known as 
    a Ringvrf, utilizing the Bandersnatch curve. Bandersnatch Ringvrf allows for a proof to be provided which simultaneously 
    guarantees the author controlled a key within a set (in our case validators), and secondly provides an output, an 
    unbiasable deterministic hash giving us a secure verifiable random function (vrf). This anonymous and secure random 
    output is a ticket and validators' tickets with the best score define the new sealing keys allowing the chosen validators 
    to exercise their privilege and create a new block at the appropriate time.
*/

use ark_vrf::reexports::ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_vrf::suites::bandersnatch::Public;
use utils::bandersnatch::Verifier;
use utils::{print_hash, log};
use ark_vrf::suites::bandersnatch::RingProofParams;

use std::sync::LazyLock;
use std::collections::VecDeque;
use std::sync::Mutex;
use sp_core::blake2_256;

use jam_types::{
    BandersnatchEpoch, BandersnatchPublic, BandersnatchRingCommitment, Block, Ed25519Public, Entropy, EntropyPool, EpochMark, OutputDataSafrole, 
    ProcessError, Safrole, SafroleErrorCode, TicketBody, TicketsMark, TicketsOrKeys, TimeSlot, ValidatorSet, ValidatorsData
};
use constants::node::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};

static VERIFIERS: LazyLock<Mutex<VecDeque<Verifier>>> = LazyLock::new(|| { Mutex::new(VecDeque::new()) });

pub mod verifier {

    use super::*;

    pub fn update(
        new_verifier: Verifier, 
        curr_validators: &ValidatorsData, 
        epoch_diff: &TimeSlot
    ) {

        let mut verifiers = VERIFIERS.lock().unwrap().clone();
        verifiers.push_back(new_verifier.clone());
        verifiers.pop_front();
        
        if *epoch_diff > 1 {
            verifiers[0] = Verifier::new(create_ring_set(curr_validators));
            verifiers[1] = new_verifier; // TODO revisar esto luego, quiza sea mejor pedir el estado completo cuando estamos offline durante un tiempo
        }
        
        set_all(verifiers);
    }

    pub fn set_all(verifiers: VecDeque<Verifier>) {
        *VERIFIERS.lock().unwrap() = verifiers;
    }

    pub fn get_all() -> VecDeque<Verifier> {
        VERIFIERS.lock().unwrap().clone()
    }

    pub fn get(validators: ValidatorSet) -> Verifier {
        match validators {
            ValidatorSet::Current => VERIFIERS.lock().unwrap().get(0).unwrap().clone(),
            ValidatorSet::Pending => VERIFIERS.lock().unwrap().get(1).unwrap().clone(),
            ValidatorSet::Next => VERIFIERS.lock().unwrap().get(2).unwrap().clone(),
            _ => VERIFIERS.lock().unwrap().get(0).unwrap().clone(), // TODO arreglar esto
        }
    } 
}

// Process Safrole state
pub fn process(
    safrole_state: &mut Safrole,
    entropy_pool: &mut EntropyPool,
    curr_validators: &mut ValidatorsData,
    prev_validators: &mut ValidatorsData,
    tau: &mut TimeSlot,
    block: &Block,
    offenders: &[Ed25519Public],
) -> Result<OutputDataSafrole, ProcessError> {

    // tau defines de most recent block
    // post_tau defines the block being processed
    let post_tau = block.header.unsigned.slot;
    log::debug!("Process Safrole state for slot {post_tau}");
    // Timeslot must be strictly monotonic
    if post_tau <= *tau {
        log::error!("Bad block slot: {:?}. The previous slot is: {:?}. Timeslot must be strictly monotonic", post_tau, tau);
        return Err(ProcessError::SafroleError(SafroleErrorCode::BadSlot));
    }
    
    // Calculate time parameters
    let epoch = *tau / EPOCH_LENGTH as TimeSlot;
    let m = *tau % EPOCH_LENGTH as TimeSlot;
    let post_epoch= post_tau / EPOCH_LENGTH as TimeSlot;
    let post_m = post_tau % EPOCH_LENGTH as TimeSlot;
    
    // Output marks
    let mut epoch_mark: Option<EpochMark> = None;
    let mut tickets_mark: Option<TicketsMark> = None;

    // The epoch mark can be some only if the block is the first in a new epoch 
    if block.header.unsigned.epoch_mark.is_some() && post_epoch == epoch {
        return Err(ProcessError::SafroleError(SafroleErrorCode::UnexpectedEpochMark));
    }
    // The winning-tickets marker is either empty or, if the block is the first after the end of the submission period for tickets and if the
    // ticket accumulator is saturated, then the final sequence of tickets identifiers.
    if block.header.unsigned.tickets_mark.is_some() &&
        (post_epoch != epoch
        || m >= TICKET_SUBMISSION_ENDS as TimeSlot
        || TICKET_SUBMISSION_ENDS as TimeSlot > post_m
        || safrole_state.ticket_accumulator.len() != EPOCH_LENGTH) 
    {
        return Err(ProcessError::SafroleError(SafroleErrorCode::UnexpectedTicketsMark));
    }
    // Check if we are in a new epoch (e' > e)
    if post_epoch > epoch {
        log::debug!("We are in a new epoch: {:?}", post_epoch);
        let mut fallback_mode = false;
        // gamma_s is the current epoch's slot-sealer series, which is either a full complement of EPOCH_LENGTH tickets
        // or, in case of fallback, a series of EPOCH_LENGTH bandersnatch keys
        if post_epoch == (epoch + 1) && m >= TICKET_SUBMISSION_ENDS as u32 && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
            // If the block signals the next epoch (by epoch index) and the previous block’s slot was within the closing period of
            // the previous epoch, then it takes the value of the prior ticket accumulator
            log::debug!("First block after the end of submission period for tickets and the ticket accumulator is saturated");
            safrole_state.seal = TicketsOrKeys::Tickets(outside_in_sequencer(&safrole_state.ticket_accumulator));
        } else if post_epoch == epoch {
            // If the block is not the first in an epoch, then it remains unchanged from the prior seal
            // gamma_s' = gamma_s
        } else {
            // Otherwise, it takes the value of the fallback key sequence
            log::warn!("Fallback mode!");
            fallback_mode = true;
        } 
        // If the block is the first in a new epoch, then a tuple of the next and current epoch randomness, along with a sequence of a tuples 
        // containing both Bandersnatch keys and Ed25519 keys for each validator defining the validator keys beginning in the next epoch
        if block.header.unsigned.epoch_mark.is_none() {
            log::error!("Empty epoch mark");
            return Err(ProcessError::SafroleError(SafroleErrorCode::EmptyEpochMark));
        }
        block::header::epoch_mark_verify(&block.header, entropy_pool)?;
        // On an epoch transition, we therefore rotate the accumulator value into the history eta1, eta2 eta3
        entropy::rotate_pool(entropy_pool);
        // With a new epoch, validator keys get rotated and the epoch's Bandersnatch key root is updated
        validators::key_rotation(safrole_state, curr_validators, prev_validators);
        // The posterior queued validator key set "pending_validators" is defined such that incoming keys belonging to the offenders 
        // are replaced with a null key containing only zeroes.
        let are_there_offenders = utils::common::set_offenders_null(&mut safrole_state.pending_validators, offenders); 
        // Create the epoch root from next pending validators and update the safrole state
        let new_verifier = if !fallback_mode {
            let new_ring_set = create_ring_set(&safrole_state.pending_validators);
            Verifier::new(new_ring_set)
        } else {
            if are_there_offenders {
                let new_ring_set = create_ring_set(&safrole_state.pending_validators);
                Verifier::new(new_ring_set)
            } else {
                verifier::get(ValidatorSet::Next)
            }            
        };
        safrole_state.epoch_root = create_root_epoch(&new_verifier);
        // Update the verifiers
        verifier::update(new_verifier, curr_validators, &(post_epoch - epoch));
        // If the block is the first in a new epoch, then a tuple of the epoch randomness and a sequence of 
        // Bandersnatch keys defining the Bandersnatch validator keys beginning in the next epoch
        log::debug!("New epoch mark");
        epoch_mark = Some(EpochMark {
            entropy: entropy_pool.buf[1].clone(),
            tickets_entropy: entropy_pool.buf[2].clone(),
            validators: {

                let bandersnatch_keys = validators::extract_keys(&safrole_state.pending_validators, |v| v.bandersnatch);
                let ed25519_keys = validators::extract_keys(&safrole_state.pending_validators, |v| v.ed25519);

                let validator_keys: [(BandersnatchPublic, Ed25519Public); VALIDATORS_COUNT] =
                    std::array::from_fn(|i| {
                        let bandersnatch_key = bandersnatch_keys[i];
                        let ed25519_key = ed25519_keys[i];
                        (bandersnatch_key, ed25519_key)
                    });

                Box::new(validator_keys)
            }
        });

        if fallback_mode {
            let bandersnatch_keys = validators::extract_keys(curr_validators,|v| v.bandersnatch);
            safrole_state.seal = TicketsOrKeys::Keys(fallback(&entropy_pool.buf[2], bandersnatch_keys));
        }

        safrole_state.ticket_accumulator = vec![];

    } else if post_epoch == epoch && m < TICKET_SUBMISSION_ENDS as u32 && TICKET_SUBMISSION_ENDS as u32 <= post_m && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
        if block.header.unsigned.tickets_mark.is_none() {
            return Err(ProcessError::SafroleError(SafroleErrorCode::EmptyTicketsMark));
        }
        // gamma_a is the ticket accumulator, a series of highestscoring ticket identifiers to be used for the next epoch.
        tickets_mark = Some(outside_in_sequencer(&safrole_state.ticket_accumulator));
        if tickets_mark != block.header.unsigned.tickets_mark {
            return Err(ProcessError::SafroleError(SafroleErrorCode::WrongTicketsMark));
        }
    }

    *tau = post_tau;

    log::debug!("Safrole state processed successfully");
    return Ok(OutputDataSafrole {epoch_mark, tickets_mark});
}

pub fn create_ring_set(validators: &ValidatorsData) -> Vec<Public> {
    log::debug!("Create ring set");
    validators
        .list
        .iter()
        .map(|v| {
            Public::deserialize_compressed_unchecked(&v.bandersnatch[..])
                // In the case a key has no corresponding Bandersnatch point when constructing the ring, then 
                // the Bandersnatch padding point as stated by Hosseini and Galassi 2024 should be substituted
                .unwrap_or_else(|_| Public::from(RingProofParams::padding_point()))
        })
        .collect()
}

pub fn create_root_epoch(verifier: &Verifier) -> BandersnatchRingCommitment {
    let mut proof: BandersnatchRingCommitment = [0u8; std::mem::size_of::<BandersnatchRingCommitment>()];
    verifier.commitment.serialize_compressed(&mut proof[..]).unwrap();
    log::debug!("Create root epoch: 0x{}", print_hash!(proof));
    return proof;
}

fn outside_in_sequencer(tickets: &[TicketBody]) -> TicketsMark {

    let mut new_ticket_accumulator = TicketsMark::default();

    for i in 0..EPOCH_LENGTH / 2 {
        new_ticket_accumulator.tickets_mark[2 * i] = tickets[i].clone();
        new_ticket_accumulator.tickets_mark[2 * i + 1] = tickets[EPOCH_LENGTH - 1 - i].clone();
    }

    return new_ticket_accumulator;
}

fn fallback(buf: &Entropy, current_keys: Box<[BandersnatchPublic; VALIDATORS_COUNT]>) -> BandersnatchEpoch {
    // This is the fallback key sequence function which selects an epoch's worth of validator Bandersnatch keys from the 
    // validator key set using the entropy collected on-chain
    let epoch_array = std::array::from_fn(|i| {
        let index_le = (i as u32).to_le_bytes();
        let hash = blake2_256(&[&buf.entropy[..], &index_le].concat());
        let hash_4 = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id = (hash_4 % VALIDATORS_COUNT as u32) as usize;
        current_keys[id]
    });

    BandersnatchEpoch {
        epoch: Box::new(epoch_array),
    }
}


