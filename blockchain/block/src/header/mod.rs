
// The header comprises a parent hash and prior state root, an extrinsic hash, a time-slot index, the epoch, winning-tickets and
// offenders markers, and, a Bandersnatch block author index and two Bandersnatch signatures; the entropy-yielding, vrf signature,
// and a block seal. Excepting the Genesis header, all block headers H have an associated parent header, whose hash is Hp.

// The epoch and winning-tickets markers are information placed in the header in order to minimize data transfer necessary to
// determine the validator keys associated with any given epoch. They are particularly useful to nodes which do not synchronize
// the entire state for any given block since they facilitate the secure tracking of changes to the validator key sets using only the
// chain of headers.

// The epoch marker specifies key and entropy relevant to the following epoch in case the ticket contest does not complete adequately
// (a very much unexpected eventuality).The epoch marker is either empty or, if the block is the first in a new epoch, then a tuple of
// the epoch randomness and a sequence of Bandersnatch keys defining the Bandersnatch validator keys (kb) beginning in the next epoch.

use bandersnatch_vrf_spec::{Prover, Verifier};
use codec::{generic_codec::encode_unsigned, {Encode, EncodeLen, EncodeSize}};
use constants::node::{EPOCH_LENGTH, TICKET_ENTRIES_PER_VALIDATOR, VALIDATORS_COUNT, TICKET_SUBMISSION_ENDS};
use jam_types::*;
use state_handler::get_state_root;
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};
use tools::{hex, log};

static PARENT_HEADER: LazyLock<Mutex<OpaqueHash>> = LazyLock::new(|| {
    Mutex::new(OpaqueHash::default())
});

pub fn get_parent_header() -> OpaqueHash {
    *PARENT_HEADER.lock().unwrap()
}

pub fn set_parent_header(parent_header: OpaqueHash) {
    log::debug!("Set parent header: {}", hex::encode(&parent_header));
    *PARENT_HEADER.lock().unwrap() = parent_header;
}

// Sealing using the ticket is of greater security, and we utilize this knowledge when determining a candidate block
// on which to extend the chain.
pub fn seal_verify(
        header: &Header,
        safrole: &Safrole,
        entropy: &EntropyPool,
        current_validators: &ValidatorsData,
        verifier: &Verifier,
) -> Result<OpaqueHash, ImportError> {
    // The header must contain a valid seal and valid vrf output. These are two signatures both using the current slot’s 
    // seal key; the message data of the former is the header’s serialization omitting the seal component Hs, whereas the 
    // latter is used as a bias-resistant entropy source and thus its message must already have been fixed: we use the entropy
    // stemming from the vrf of the seal signature. 
    let unsigned_header = header.unsigned.encode();
    // Get the block author
    let block_author = header.unsigned.author_index as usize;
    let i = header.unsigned.slot % EPOCH_LENGTH as TimeSlot;

    let seal_vrf_output = match &safrole.seal {
        Seal::Tickets(tickets) => {
            log::debug!("Verify tickets seal");
            // The context is "jam_fallback_seal" + entropy[3] + ticket_attempt
            let context = [&b"jam_ticket_seal"[..], &entropy.buf[3].encode(), &tickets.tickets_mark[i as usize].attempt.encode()].concat();
            // Verify the seal
            let seal_vrf_output_result = verifier.ietf_vrf_verify(
                                                    &context,
                                                    &unsigned_header,
                                                    &header.seal,
                                                    block_author,
            );

            let seal_vrf_output = match seal_vrf_output_result {
                Ok(vrf_output) => vrf_output,
                Err(_) => {
                    log::error!("Invalid tickets seal");
                    return Err(ImportError::SafroleError(SafroleErrorCode::InvalidTicketSeal));
                }
            };

            if tickets.tickets_mark[i as usize].id != seal_vrf_output {
                log::error!("Ticket {i} not match: id {} != seal vrf {}", tools::print_hash!(tickets.tickets_mark[i as usize].id), tools::print_hash!(seal_vrf_output));
                return Err(ImportError::SafroleError(SafroleErrorCode::TicketNotMatch));
            }
            log::debug!("Seal tickets verified successfully");
            seal_vrf_output
        },
        Seal::Keys(keys) => {
            log::debug!("Verify keys seal");
            // The context is "jam_fallback_seal" + entropy[3]
            let context = [&b"jam_fallback_seal"[..], &entropy.buf[3].encode()].concat();
            
            // Verify the seal
            let seal_vrf_output_result = verifier.ietf_vrf_verify(
                                                        &context,
                                                        &unsigned_header,
                                                        &header.seal,
                                                        block_author,
            );
            let seal_vrf_output = match seal_vrf_output_result {
                Ok(vrf_output) => vrf_output,
                Err(_) => {
                    log::error!("Invalid key seal");
                    return Err(ImportError::SafroleError(SafroleErrorCode::InvalidTicketSeal));
                }
            };
            
            if keys.epoch[i as usize] != current_validators.list[block_author].bandersnatch {
                log::error!("Key not match: Seal key {:02x?} != bandersnatch key author {block_author} {:02x?}", tools::print_hash!(keys.epoch[i as usize]), tools::print_hash!(current_validators.list[block_author].bandersnatch));
                return Err(ImportError::SafroleError(SafroleErrorCode::KeyNotMatch));
            }

            log::debug!("Seal keys verified successfully");
            seal_vrf_output
        },
        Seal::None => {
            log::error!("None tickets or keys");
            return Err(ImportError::SafroleError(SafroleErrorCode::SealNone));
        },
    };
    
    // Verify the entropy source
    let context = [&b"jam_entropy"[..], &seal_vrf_output.encode()].concat();
    let entropy_source_vrf_result = verifier.ietf_vrf_verify(
                                                                            &context,
                                                                            &[],
                                                                            &header.unsigned.entropy_source,
                                                                            block_author);
    let entropy_source_vrf_output = match entropy_source_vrf_result {
        Ok(_) => entropy_source_vrf_result.unwrap(),
        Err(_) => { 
            log::error!("Invalid entropy source");
            return Err(ImportError::SafroleError(SafroleErrorCode::InvalidEntropySource)) 
        },
    };

    log::debug!("Seal header verified successfully. vrf output: 0x{}", tools::print_hash!(entropy_source_vrf_output));
    Ok(entropy_source_vrf_output)
}

pub fn epoch_mark_verify(header: &Header, entropy_pool: &EntropyPool) -> Result<(), ImportError> {

    if header.unsigned.epoch_mark.as_ref().unwrap().entropy != entropy_pool.buf[0] {
        log::error!("Entropy epoch mark doesn't match with η0");
        return Err(ImportError::SafroleError(SafroleErrorCode::WrongEpochMark));
    }

    if header.unsigned.epoch_mark.as_ref().unwrap().tickets_entropy != entropy_pool.buf[1] {
        log::error!("Tickets entropy doesn't match with η1");
        return Err(ImportError::SafroleError(SafroleErrorCode::WrongEpochMark));
    }

    let next_validators = state_handler::validators::get(ValidatorSet::Next);

    for (i, validator) in next_validators.list.iter().enumerate() {

        if validator.bandersnatch != header.unsigned.epoch_mark.as_ref().unwrap().validators[i].0
        || validator.ed25519 != header.unsigned.epoch_mark.as_ref().unwrap().validators[i].1 {
            log::error!("Post pending validator index {:?} doesn't match with the one in the epoch mark", i);
            return Err(ImportError::SafroleError(SafroleErrorCode::WrongEpochMark));
        } 
    }

    return Ok(())
}

pub fn verify(block: &Block) -> Result<(), ImportError> {

    tickets_verify(&block.header)?;
    extrinsic_verify(&block)?;
    validator_index_verify(&block.header)?;
    offenders_verify(&block)?;
    state_root_verify(&block.header)?;   
    parent_header_verify(&block.header)?;
    
    log::debug!("Header verified successfully");
    return Ok(());
}

fn tickets_verify(header: &Header) -> Result<(), ImportError> {

    if header.unsigned.tickets_mark.is_some() {

        for i in 0..EPOCH_LENGTH {

            if header.unsigned.tickets_mark.as_ref().unwrap().tickets_mark[i].attempt >= TICKET_ENTRIES_PER_VALIDATOR {
                log::error!("Ticket mark {:?} has an attempt {:?} >= Max tickets entries per validator {:?}", i, header.unsigned.tickets_mark.as_ref().unwrap().tickets_mark[i].attempt, TICKET_ENTRIES_PER_VALIDATOR);
                return Err(ImportError::HeaderError(HeaderErrorCode::BadTicketAttempt));
            }
        }
    }
    Ok(())
}

pub fn offenders_verify(block: &Block) -> Result<(), ImportError> {
    
    let mut extrinsic_offenders: HashSet<Ed25519Public> = HashSet::new();

    let _ = block.extrinsic.disputes.culprits.iter().map(|k| extrinsic_offenders.insert(k.key));
    let _ = block.extrinsic.disputes.faults.iter().map(|k| extrinsic_offenders.insert(k.key));
    
    //The offenders markers must contain exactly the keys of all new offenders, respectively
    for header_offender in &block.header.unsigned.offenders_mark {
        if !extrinsic_offenders.contains(header_offender) {
            log::error!("The offender {} is found in the header, but is not found in the extrinsic", tools::print_hash!(*header_offender));
            return Err(ImportError::HeaderError(HeaderErrorCode::BadOffenders));
        }
    }

    if extrinsic_offenders.len() != block.header.unsigned.offenders_mark.len() {
        log::error!("The amount of offenders in the extrinsic {:?} not match with the amount of the offenders in the header {:?}",
                                                                extrinsic_offenders.len(), block.header.unsigned.offenders_mark.len());
        return Err(ImportError::HeaderError(HeaderErrorCode::BadOffenders));
    }

    return Ok(());
}

fn parent_header_verify(header: &Header) -> Result<(), ImportError> {

    let parent_header = get_parent_header();

    if parent_header != header.unsigned.parent {
        log::error!("Expected parent header {} != received parent header {}", tools::print_hash!(parent_header), tools::print_hash!(header.unsigned.parent));
        return Err(ImportError::HeaderError(HeaderErrorCode::BadParentHeader));
    }

    return Ok(());
}

pub fn state_root_verify(header: &Header) -> Result<(), ImportError> {

    let parent_state_root = get_state_root().lock().unwrap();

    if header.unsigned.parent_state_root != *parent_state_root {
        log::error!("Bad parent state root: header state root {} != parent state root {}", tools::print_hash!(header.unsigned.parent_state_root), tools::print_hash!(*parent_state_root));
        return Err(ImportError::HeaderError(HeaderErrorCode::BadParentStateRoot));
    }

    log::debug!("The block's parent state root {} matches", tools::print_hash!(header.unsigned.parent_state_root));
    return Ok(());
}

pub fn validator_index_verify(header: &Header) -> Result<(), ImportError> { 

    if header.unsigned.author_index >= VALIDATORS_COUNT as u16 {
        log::error!("Bad validator index: {:?}. The total number of validators is {:?}", header.unsigned.author_index, VALIDATORS_COUNT);
        return Err(ImportError::HeaderError(HeaderErrorCode::BadValidatorIndex));
    }

    return Ok(());
}

pub fn extrinsic_verify(block: &Block) -> Result<(), ImportError> {

    let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>() * block.extrinsic.guarantees.len());
    encode_unsigned(block.extrinsic.guarantees.len()).encode_to(&mut guarantees_blob);

    for guarantee in block.extrinsic.guarantees.iter() {

        sp_core::blake2_256(&guarantee.report.encode()).encode_to(&mut guarantees_blob);
        guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
        encode_unsigned(guarantee.signatures.len()).encode_to(&mut guarantees_blob);

        for signature in &guarantee.signatures {
            signature.validator_index.encode_to(&mut guarantees_blob);
            signature.signature.encode_to(&mut guarantees_blob);
        }
    }

    let a = [sp_core::blake2_256(&block.extrinsic.tickets.encode_len()),
                            sp_core::blake2_256(&block.extrinsic.preimages.encode_len()),
                            sp_core::blake2_256(&guarantees_blob),
                            sp_core::blake2_256(&block.extrinsic.assurances.encode_len()),
                            sp_core::blake2_256(&block.extrinsic.disputes.encode())].concat();

    if block.header.unsigned.extrinsic_hash != sp_core::blake2_256(&a) {
        log::error!("Bad extrinsic hash: header extrinsic hash {} != calculated {}", tools::print_hash!(block.header.unsigned.extrinsic_hash), tools::print_hash!(sp_core::blake2_256(&a)));
        return Err(ImportError::HeaderError(HeaderErrorCode::BadExtrinsicHash));
    }

    log::trace!("Header extrinsic expected: {:x?}", block.header.unsigned.extrinsic_hash );
    log::trace!("Header extrinsic   result: {:x?}\n", sp_core::blake2_256(&a) );
    
    return Ok(());
}

pub fn encode_extrinsic(block: &Block) -> OpaqueHash {

    let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Header>() * block.extrinsic.guarantees.len());
    encode_unsigned(block.extrinsic.guarantees.len()).encode_to(&mut guarantees_blob);

    for guarantee in block.extrinsic.guarantees.iter() {

        sp_core::blake2_256(&guarantee.report.encode()).encode_to(&mut guarantees_blob);
        guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
        encode_unsigned(guarantee.signatures.len()).encode_to(&mut guarantees_blob);

        for signature in &guarantee.signatures {
            signature.validator_index.encode_to(&mut guarantees_blob);
            signature.signature.encode_to(&mut guarantees_blob);
        }
    }

    let a = [sp_core::blake2_256(&block.extrinsic.tickets.encode_len()),
                            sp_core::blake2_256(&block.extrinsic.preimages.encode_len()),
                            sp_core::blake2_256(&guarantees_blob),
                            sp_core::blake2_256(&block.extrinsic.assurances.encode_len()),
                            sp_core::blake2_256(&block.extrinsic.disputes.encode())].concat();
    
    sp_core::blake2_256(&a)
}

// Predict what the seal will be for the next epoch, using pre-rotation values. This allows a node to determine block authorship 
// at epoch boundaries without waiting for the epoch transition to complete.
pub fn predict_next_epoch_seal(
    safrole_state: &Safrole,
    entropy_pre_rotation: &Entropy,
    m: TimeSlot,
) -> Seal {
    if m >= TICKET_SUBMISSION_ENDS as TimeSlot && safrole_state.ticket_accumulator.len() == EPOCH_LENGTH {
        Seal::Tickets(outside_in_sequencer(&safrole_state.ticket_accumulator))
    } else {
        let bandersnatch_keys = validators::extract_keys(&safrole_state.pending_validators, |v| v.bandersnatch);
        Seal::Keys(fallback(entropy_pre_rotation, bandersnatch_keys))
    }
}

pub fn get_seal(state: &GlobalState, current_slot: TimeSlot) -> Option<Seal> {

    let prev_slot = state_handler::time::get();
    let prev_epoch = prev_slot / EPOCH_LENGTH as TimeSlot;
    let epoch = current_slot / EPOCH_LENGTH as TimeSlot;

    if epoch == prev_epoch + 1 {
        // Clear the ticket pool
        crate::extrinsic::tickets::clear_pool();
        // New epoch: predict seal from pre-rotation state
        let m = prev_slot % EPOCH_LENGTH as TimeSlot;
        log::debug!("Epoch boundary: predicting seal for epoch {} (tau={}, m={})", epoch, prev_slot, m);
        Some(predict_next_epoch_seal(&state.safrole, &state.entropy.buf[1], m))
    } else if epoch == prev_epoch {
        // Same epoch: use current seal
        Some(state.safrole.seal.clone())
    } else {
        // Multiple epochs behind or ahead — don't produce // TODO solicit state update
        crate::extrinsic::tickets::clear_pool();
        log::debug!("Multiple epochs behind or ahead: current epoch={} prev epoch={}, skipping production", epoch, prev_epoch);
        return None;
    }
}

pub fn seal_winning_verify(
    state: &GlobalState,
    seal: Seal,
    current_slot: TimeSlot,
    prover: &Prover,
    our_bandersnatch_public: &BandersnatchPublic,
) -> bool 
{
    let m = (current_slot % EPOCH_LENGTH as TimeSlot) as usize;

     match seal {
        Seal::Tickets(ref tickets) => {
            let ticket = &tickets.tickets_mark[m];
            log::info!("Seal mode: Tickets. Slot {} ticket id: {}", current_slot, hex::encode(&ticket.id));

            let context = [&b"jam_ticket_seal"[..], &state.entropy.buf[3].encode(), &ticket.attempt.encode()].concat();
            let our_vrf_output: Vec<u8> = prover.vrf_output(&context);

            if our_vrf_output == ticket.id {
                log::info!("Block production: ticket matches! Slot {}", current_slot);
                return true;
            }
        },
        Seal::Keys(ref keys) => {
            let key = &keys.epoch[m];
            log::info!("Seal mode: Keys (fallback). Slot {} key: {}", current_slot, hex::encode(key));

            if *key == *our_bandersnatch_public {
                log::info!("Block production: key matches! Slot {}", current_slot);
                return true;
            }
        },
        Seal::None => { },
    }

    return false;
}