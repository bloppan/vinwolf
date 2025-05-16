use sp_core::blake2_256;
use crate::blockchain::state::reporting_assurance::{add_assignment, remove_assignment};
use crate::types::{
    AssurancesErrorCode, OutputDataAssurances, ValidatorIndex, AssurancesExtrinsic, Hash, TimeSlot, 
    AvailabilityAssignment, CoreIndex, AvailabilityAssignments, ProcessError, ValidatorSet
};
use crate::constants::{AVAIL_BITFIELD_BYTES, CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY};
use crate::blockchain::state::get_validators;
use crate::utils::codec::Encode;
use crate::utils::common::{is_sorted_and_unique, VerifySignature};

// The assurances extrinsic are the input data of workloads they have correctly received and are storing locally.
// The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a 
// sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the 
// validator who is assuring. A value of 1 at any given index implies that the validator assures they are contributing 
// to its availability.

impl AssurancesExtrinsic {
    
    pub fn process(
        &self, 
        assurances_state: &mut AvailabilityAssignments, 
        post_tau: &TimeSlot, 
        parent: &Hash) 
    -> Result<OutputDataAssurances, ProcessError> {

        // TODO no se si poner esto
        /*if self.assurances.is_empty() {
            return Ok(OutputDataAssurances { reported: Vec::new() });
        }*/

        // The assurances extrinsic is a sequence of assurance values, at most one per validator
        if self.assurances.len() > VALIDATORS_COUNT {
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::TooManyAssurances));
        }

        // The assurances must all ordered by validator index
        let validator_indexes = self.assurances.iter()
                                                            .map(|assurance| assurance.validator_index)
                                                            .collect::<Vec<ValidatorIndex>>();

        // The index can not be greater or equal than the total number of validators
        for index in &validator_indexes {
            if *index as usize >= VALIDATORS_COUNT {
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadValidatorIndex));
            }
        }                       

        // The assurances must all be ordered by validator index
        if !is_sorted_and_unique(&validator_indexes) {
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::NotSortedOrUniqueAssurers));
        }
        
        let current_validators = get_validators(ValidatorSet::Current);
        //let list = get_reporting_assurance();
        let mut core_marks = [0_usize; CORES_COUNT as usize];
        for assurance in &self.assurances {
            // The assurances must all be anchored on the parent
            if assurance.anchor != *parent {
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadAttestationParent));
            }
            // The signature must be one whose public key is that of the validator assuring and whose message is the
            // serialization of the parent hash and the aforementioned bitstring.
            let mut message = Vec::from(b"jam_available");
            let mut serialization = Vec::new();
            parent.encode_to(&mut serialization);
            assurance.bitfield.encode_to(&mut serialization);
            message.extend_from_slice(&blake2_256(&serialization));
            let validator = &current_validators[assurance.validator_index as usize]; 
            if !assurance.signature.verify_signature(&message, &validator.ed25519) {
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadSignature));
            }
            if assurance.bitfield.len() != AVAIL_BITFIELD_BYTES {
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::WrongBitfieldLength));
            }
            
            for core in 0..CORES_COUNT {
                let bitfield = assurance.bitfield[core / 8] & (1 << core % 8) != 0;
                if bitfield {
                    // A bit may only be set if the corresponding core has a report pending availability on it
                    if assurances_state[core as usize].is_none() {
                        return Err(ProcessError::AssurancesError(AssurancesErrorCode::CoreNotEngaged));
                    }
                    core_marks[core as usize] += 1;
                }
            }
        }

        // A work-report is said to become available if and only if there are a clear super-majority of validators
        // who have marked its core as set within the block's assurance extrinsic. We define the sequence of newly
        // available work-reports in the next reported vector.
        let mut reported = Vec::new();
        let mut to_remove = Vec::new();
        for core in 0..CORES_COUNT {
            if core_marks[core as usize] >= VALIDATORS_SUPER_MAJORITY {
                if let Some(assignment) = &assurances_state[core as usize] {
                    reported.push(assignment.report.clone());
                    to_remove.push(core as CoreIndex);
                }
            }
        }

        // The Availability Assignments are equivalents except for the removal of items which are now available
        for core in &to_remove {
            if let Some(assignment) = &assurances_state[*core as usize] {
                add_assignment(&AvailabilityAssignment {
                    report: assignment.report.clone(),
                    timeout: post_tau.clone(),
                }, assurances_state);
            }
        }

        for core in to_remove {
            remove_assignment(&core, assurances_state);
        }
    
        Ok(OutputDataAssurances { reported } )
    }

}


