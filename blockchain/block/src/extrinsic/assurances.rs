/*
    The Assurances Extrinsic. The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a sequence 
    of binary values (i.e. a bitstring), one per core, together with a signature and the index of the validator who is assuring. A value of 1
    (or ⊺, if interpreted as a Boolean) at any given index implies that the validator assures they are contributing to its availability.
*/

use sp_core::blake2_256;
use jam_types::{
    AssurancesErrorCode, OutputDataAssurances, ValidatorIndex, Hash, TimeSlot, Assurance, CoreIndex, AvailabilityAssignments, 
    ProcessError, ValidatorSet
};
use constants::node::{AVAIL_BITFIELD_BYTES, CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY, REPORTED_WORK_REPLACEMENT_PERIOD};
use codec::Encode;
use utils::common::{is_sorted_and_unique, VerifySignature};

pub fn process(
    assurances_extrinsic: &[Assurance],
    availability_state: &mut AvailabilityAssignments, 
    post_tau: &TimeSlot, 
    parent: &Hash) 
-> Result<OutputDataAssurances, ProcessError> {

    log::debug!("Processing assurances extrinsic...");
    // TODO no se si poner esto
    /*if self.assurances.is_empty() {
        return Ok(OutputDataAssurances { reported: Vec::new() });
    }*/
    // The assurances extrinsic is a sequence of assurance values, at most one per validator
    if assurances_extrinsic.len() > VALIDATORS_COUNT {
        log::error!("Too many extrinsic assurances: {:?}", assurances_extrinsic.len());
        return Err(ProcessError::AssurancesError(AssurancesErrorCode::TooManyAssurances));
    }

    // The assurances must all ordered by validator index
    let validator_indexes = assurances_extrinsic.iter()
                                                        .map(|assurance| assurance.validator_index)
                                                        .collect::<Vec<ValidatorIndex>>();

    // The index can not be greater or equal than the total number of validators
    for index in &validator_indexes {
        if *index as usize >= VALIDATORS_COUNT {
            log::error!("Bad validator index: {:?}", *index);
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadValidatorIndex));
        }
    }                       

    // The assurances must all be ordered by validator index
    if !is_sorted_and_unique(&validator_indexes) {
        log::error!("Not sorted or unique assurers");
        return Err(ProcessError::AssurancesError(AssurancesErrorCode::NotSortedOrUniqueAssurers));
    }
    
    let current_validators = state_handler::validators::get(ValidatorSet::Current);
    //let list = get_reporting_assurance();
    let mut core_marks = [0_usize; CORES_COUNT as usize];
    for assurance in assurances_extrinsic {
        // The assurances must all be anchored on the parent
        if assurance.anchor != *parent {
            log::error!("Bad attestation parent: 0x{} != Block parent: 0x{}", utils::print_hash!(assurance.anchor), utils::print_hash!(*parent));
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadAttestationParent));
        }
        // The signature must be one whose public key is that of the validator assuring and whose message is the
        // serialization of the parent hash and the aforementioned bitstring.
        let mut message = Vec::from(b"jam_available");
        let mut serialization = Vec::new();
        parent.encode_to(&mut serialization);
        assurance.bitfield.encode_to(&mut serialization);
        message.extend_from_slice(&blake2_256(&serialization));
        let validator = &current_validators.list[assurance.validator_index as usize]; 
        if !assurance.signature.verify_signature(&message, &validator.ed25519) {
            log::error!("Bad signature in validator index {:?}", assurance.validator_index);
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadSignature));
        }
        if assurance.bitfield.len() != AVAIL_BITFIELD_BYTES {
            log::error!("Wrong bitfield length {:?} != available bitfield bytes {:?}", assurance.bitfield.len(), AVAIL_BITFIELD_BYTES);
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::WrongBitfieldLength));
        }
        
        for core in 0..CORES_COUNT {
            let bitfield = assurance.bitfield[core / 8] & (1 << core % 8) != 0;
            if bitfield {
                // A bit may only be set if the corresponding core has a report pending availability on it
                if availability_state.list[core as usize].is_none() {
                    log::error!("Core {:?} not engaged", core);
                    return Err(ProcessError::AssurancesError(AssurancesErrorCode::CoreNotEngaged));
                }
                core_marks[core as usize] += 1;
            }
        }
    }

    // A work-report is said to become available if and only if there are a clear super-majority of validators
    // who have marked its core as set within the block's assurance extrinsic. We define the sequence of newly
    // available work-reports in the new_availables_wr vector.
    let mut new_availables_wr = Vec::new();

    for core in 0..CORES_COUNT {
        if let Some(assignment) = &availability_state.list[core as usize] {
            // We remove the items which are available
            if core_marks[core as usize] >= VALIDATORS_SUPER_MAJORITY {
                new_availables_wr.push(assignment.report.clone());
                log::debug!("Remove assignment {} from core {:?}. This report is now available", utils::print_hash!(assignment.report.package_spec.hash), core);
                state_handler::reports::remove_assignment(&(core as CoreIndex), availability_state);
            // We also remove the items which have timed out
            } else if *post_tau >= assignment.timeout + REPORTED_WORK_REPLACEMENT_PERIOD as TimeSlot {
                log::debug!("Remove assignment {} from core {:?}. This report have timed out!", utils::print_hash!(assignment.report.package_spec.hash), core);
                state_handler::reports::remove_assignment(&(core as CoreIndex), availability_state);
            }
        };
    }
    
    log::debug!("Assurances extrinsic processed successfully");

    Ok(OutputDataAssurances { reported: new_availables_wr } )
}




