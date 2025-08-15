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
use utils::print_hash;
use ark_vrf::suites::bandersnatch::RingProofParams;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use sp_core::blake2_256;

use jam_types::{
    BandersnatchEpoch, BandersnatchPublic, BandersnatchRingCommitment, Ed25519Public, Entropy, EntropyPool, EpochMark, Block,
    OutputDataSafrole, Safrole, SafroleErrorCode, TicketBody, ProcessError, TicketsMark, TicketsOrKeys, TimeSlot, ValidatorsData
};
use block::{extrinsic, header};
use constants::node::{VALIDATORS_COUNT, EPOCH_LENGTH, TICKET_SUBMISSION_ENDS};
use codec::Encode;

static RING_SET: Lazy<Mutex<Option<(u32, Vec<Public>)>>> = Lazy::new(|| { Mutex::new(None) });

fn set_ring_set(epoch: u32, ring_set: Vec<Public>) {
    let mut ring_set_lock = RING_SET.lock().unwrap();
    *ring_set_lock = Some((epoch, ring_set));
}

fn _get_ring_set_cached() -> Option<(u32, Vec<Public>)> {
    let ring_set_lock = RING_SET.lock().unwrap();
    ring_set_lock.as_ref().map(|(epoch, vec)| (*epoch, vec.clone()))
}

fn _get_ring_set(epoch: u32, validators: &ValidatorsData) -> Vec<Public> {

    let ring_set_cached = _get_ring_set_cached();

    if ring_set_cached.is_none() {
        log::debug!("The ring set is none... Create new ring");
        let new_ring_set = create_ring_set(&validators);
        set_ring_set(epoch, new_ring_set.clone());
        return new_ring_set;
    }

    let epoch_ring_set_cached = ring_set_cached.as_ref().unwrap().0;
    let ring_set_cached = ring_set_cached.unwrap().1;

    if epoch_ring_set_cached != epoch {
        log::debug!("The ring set catched was created in a different epoch... Create new ring");
        let new_ring_set = create_ring_set(&validators);
        set_ring_set(epoch, new_ring_set.clone());
        return new_ring_set;
    }

    return ring_set_cached;
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

    log::debug!("Process Safrole state for slot {tau}");
    // tau defines de most recent block
    // post_tau defines the block being processed
    let post_tau = block.header.unsigned.slot;
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

    // Check if we are in a new epoch (e' > e)
    if post_epoch > epoch {
        log::info!("We are in a new epoch: {:?}", post_epoch);
        // On an epoch transition, we therefore rotate the accumulator value into the history eta1, eta2 eta3
        entropy::rotate_pool(entropy_pool);
        // With a new epoch, validator keys get rotated and the epoch's Bandersnatch key root is updated
        validators::key_rotation(safrole_state, curr_validators, prev_validators, offenders);
        // Create the epoch root from next pending validators and update the safrole state
        safrole_state.epoch_root = create_root_epoch(create_ring_set(&safrole_state.pending_validators));
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
        // gamma_s is the current epoch's slot-sealer series, which is either a full complement of EPOCH_LENGTH tickets
        // or, in case of fallback, a series of EPOCH_LENGTH bandersnatch keys
        if post_epoch == (epoch + 1) && m >= TICKET_SUBMISSION_ENDS as u32 && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
            // If the block is the first after the end of the submission period for tickets and if the ticket accumulator 
            // is saturated, then the final sequence of ticket identifiers
            log::debug!("First block after the end of submission period for tickets and the ticket accumulator is saturated");
            safrole_state.seal = TicketsOrKeys::Tickets(outside_in_sequencer(&safrole_state.ticket_accumulator));
        } else if post_epoch == epoch {
            // gamma_s' = gamma_s
        } else {
            // Otherwise, it takes the value of the fallback key sequence
            log::warn!("Fallback mode!");
            let bandersnatch_keys = validators::extract_keys(curr_validators,|v| v.bandersnatch);
            safrole_state.seal = TicketsOrKeys::Keys(fallback(&entropy_pool.buf[2], &bandersnatch_keys));
        } 
        safrole_state.ticket_accumulator = vec![];
    } else if post_epoch == epoch && m < TICKET_SUBMISSION_ENDS as u32 && TICKET_SUBMISSION_ENDS as u32 <= post_m && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
        // gamma_a is the ticket accumulator, a series of highestscoring ticket identifiers to be used for the next epoch.
        tickets_mark = Some(outside_in_sequencer(&safrole_state.ticket_accumulator));
    }
    
    /*// Get the ring set // TODO after M1
    let ring_set = get_ring_set(post_epoch, &safrole_state.pending_validators);*/
    let curr_val_ring_set = create_ring_set(&curr_validators);
    let pending_val_ring_set = create_ring_set(&safrole_state.pending_validators);
    // Process tickets extrinsic
    extrinsic::tickets::process(&block.extrinsic.tickets, safrole_state, entropy_pool, &post_tau, pending_val_ring_set)?;
    // update tau which defines the most recent block's index
    *tau = post_tau;
    // Verify the header's seal
    let entropy_source_vrf_output = header::seal_verify(&block.header, &safrole_state, &entropy_pool, &curr_validators, curr_val_ring_set)?;
    // Update recent entropy eta0
    entropy::update_recent(entropy_pool, entropy_source_vrf_output);
    
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

pub fn create_root_epoch(ring_set: Vec<Public>) -> BandersnatchRingCommitment {
    let verifier = Verifier::new(ring_set);
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

fn fallback(buf: &Entropy, current_keys: &Box<[BandersnatchPublic; VALIDATORS_COUNT]>) -> BandersnatchEpoch {
    // This is the fallback key sequence function which selects an epoch's worth of validator Bandersnatch keys from the 
    // validator key set using the entropy collected on-chain
    let mut new_epoch_keys: BandersnatchEpoch = BandersnatchEpoch::default();
    for i in 0u32..EPOCH_LENGTH as u32 { 
        let index_le = (i as u32).encode();
        let hash = blake2_256(&[&buf.entropy[..], &index_le].concat());
        let hash_4 = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let id = (hash_4 % VALIDATORS_COUNT as u32) as usize;
        new_epoch_keys.epoch[i as usize] = current_keys[id].clone();
    }

    return new_epoch_keys;
}


