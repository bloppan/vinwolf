use sp_core::blake2_256;
use handler::{add_assignment, remove_assignment, get_validators};
use crate::AssurancesExtrinsic;
use jam_types::{
    AssurancesErrorCode, OutputDataAssurances, ValidatorIndex, Hash, TimeSlot, ReadError, AvailAssurance, OpaqueHash, Ed25519Signature,
    AvailabilityAssignment, CoreIndex, AvailabilityAssignments, ProcessError, ValidatorSet
};
use constants::node::{AVAIL_BITFIELD_BYTES, CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY};
use codec::{Encode, EncodeSize, Decode, BytesReader};
use codec::generic_codec::{encode_unsigned, decode_unsigned};
use utils::common::{is_sorted_and_unique, VerifySignature};

impl Default for AssurancesExtrinsic {
    fn default() -> Self {
        AssurancesExtrinsic {
            assurances: Vec::new(),
        }
    }
}

impl Encode for AssurancesExtrinsic {
    
    fn encode(&self) -> Vec<u8> {

        let mut assurances_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<AssurancesExtrinsic>() * self.assurances.len());
        encode_unsigned(self.assurances.len()).encode_to(&mut assurances_blob);

        for assurance in &self.assurances {
            assurance.anchor.encode_to(&mut assurances_blob);
            assurance.bitfield.encode_to(&mut assurances_blob);
            assurance.validator_index.encode_size(2).encode_to(&mut assurances_blob);
            assurance.signature.encode_to(&mut assurances_blob);
        }

        return assurances_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for AssurancesExtrinsic {

    fn decode(assurances_blob: &mut BytesReader) -> Result<Self, ReadError> {

        let num_assurances = decode_unsigned(assurances_blob)?;
        let mut assurances = Vec::with_capacity(num_assurances);  

        for _ in 0..num_assurances {
            assurances.push(AvailAssurance {
                anchor: OpaqueHash::decode(assurances_blob)?,
                bitfield: <[u8; AVAIL_BITFIELD_BYTES]>::decode(assurances_blob)?,
                validator_index: ValidatorIndex::decode(assurances_blob)?,
                signature: Ed25519Signature::decode(assurances_blob)?,
            });
        }
        
        Ok(AssurancesExtrinsic { assurances })
    }
}

impl AssurancesExtrinsic {
    
    pub fn process(
        &self, 
        assurances_state: &mut AvailabilityAssignments, 
        post_tau: &TimeSlot, 
        parent: &Hash) 
    -> Result<OutputDataAssurances, ProcessError> {

        log::debug!("Processing assurances extrinsic...");
        // TODO no se si poner esto
        /*if self.assurances.is_empty() {
            return Ok(OutputDataAssurances { reported: Vec::new() });
        }*/
        // The assurances extrinsic is a sequence of assurance values, at most one per validator
        if self.assurances.len() > VALIDATORS_COUNT {
            log::error!("Too many extrinsic assurances: {:?}", self.assurances.len());
            return Err(ProcessError::AssurancesError(AssurancesErrorCode::TooManyAssurances));
        }

        // The assurances must all ordered by validator index
        let validator_indexes = self.assurances.iter()
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
        
        let current_validators = get_validators(ValidatorSet::Current);
        //let list = get_reporting_assurance();
        let mut core_marks = [0_usize; CORES_COUNT as usize];
        for assurance in &self.assurances {
            // The assurances must all be anchored on the parent
            if assurance.anchor != *parent {
                log::error!("Bad attestation parent: 0x{} Block parent: 0x{}", utils::print_hash!(assurance.anchor), utils::print_hash!(*parent));
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
                log::error!("Bad signature");
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::BadSignature));
            }
            if assurance.bitfield.len() != AVAIL_BITFIELD_BYTES {
                log::error!("Wrong bitfield length");
                return Err(ProcessError::AssurancesError(AssurancesErrorCode::WrongBitfieldLength));
            }
            
            for core in 0..CORES_COUNT {
                let bitfield = assurance.bitfield[core / 8] & (1 << core % 8) != 0;
                if bitfield {
                    // A bit may only be set if the corresponding core has a report pending availability on it
                    if assurances_state.list[core as usize].is_none() {
                        log::error!("Core {:?} not engaged", core);
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
                if let Some(assignment) = &assurances_state.list[core as usize] {
                    reported.push(assignment.report.clone());
                    to_remove.push(core as CoreIndex);
                }
            }
        }

        // The Availability Assignments are equivalents except for the removal of items which are now available
        for core in &to_remove {
            if let Some(assignment) = &assurances_state.list[*core as usize] {
                log::debug!("New assignment 0x{} added to core {:?} with timeout: {:?}", utils::print_hash!(assignment.report.package_spec.hash), *core, post_tau);
                add_assignment(&AvailabilityAssignment {
                    report: assignment.report.clone(),
                    timeout: post_tau.clone(),
                }, assurances_state);
            }
        }

        for core in to_remove {
            log::debug!("Remove assignment from core {:?}", core);
            remove_assignment(&core, assurances_state);
        }
        
        log::debug!("Assurances extrinsic processed successfully");

        Ok(OutputDataAssurances { reported } )
    }

}


