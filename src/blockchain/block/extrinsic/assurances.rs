use sp_core::blake2_256;
use crate::blockchain::state::reporting_assurance::{add_assignment, remove_assignment};
use crate::types::{
    OpaqueHash, Ed25519Signature, ValidatorIndex, AssurancesExtrinsic, AvailAssurance, WorkReport, Hash, TimeSlot,
    AvailabilityAssignment, CoreIndex
};
use crate::constants::{AVAIL_BITFIELD_BYTES, CORES_COUNT, VALIDATORS_COUNT, VALIDATORS_SUPER_MAJORITY};
use crate::blockchain::state::validators::{get_validators_state, ValidatorSet};
use crate::blockchain::state::get_reporting_assurance_state;
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};
use crate::utils::common::{is_sorted_and_unique, VerifySignature};

// The assurances extrinsic are the input data of workloads they have correctly received and are storing locally.
// The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a 
// sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the 
// validator who is assuring. A value of 1 at any given index implies that the validator assures they are contributing 
// to its availability.

impl AssurancesExtrinsic {
    
    pub fn process(&self, post_tau: &TimeSlot, parent: &Hash) -> Result<OutputDataAssurances, ErrorCode> {

        // TODO no se si poner esto
        /*if self.assurances.is_empty() {
            return Ok(OutputDataAssurances { reported: Vec::new() });
        }*/

        // The assurances extrinsic is a sequence of assurance values, at most one per validator
        if self.assurances.len() > VALIDATORS_COUNT {
            return Err(ErrorCode::TooManyAssurances);
        }

        // The assurances must all ordered by validator index
        let validator_indexes = self.assurances.iter()
                                                            .map(|assurance| assurance.validator_index)
                                                            .collect::<Vec<ValidatorIndex>>();

        // The index can not be greater or equal than the total number of validators
        for index in &validator_indexes {
            if *index as usize >= VALIDATORS_COUNT {
                return Err(ErrorCode::BadValidatorIndex);
            }
        }                       

        // The assurances must all be ordered by validator index
        if !is_sorted_and_unique(&validator_indexes) {
            return Err(ErrorCode::NotSortedOrUniqueAssurers);
        }

        let current_validators = get_validators_state(ValidatorSet::Current);
        let list = get_reporting_assurance_state();
        let mut core_marks = [0_usize; CORES_COUNT as usize];
        for assurance in &self.assurances {
            // The assurances must all be anchored on the parent
            if assurance.anchor != *parent {
                return Err(ErrorCode::BadAttestationParent);
            }
            // The signature must be one whose public key is that of the validator assuring and whose message is the
            // serialization of the parent hash and the aforementioned bitstring.
            let mut message = Vec::from(b"jam_available");
            let mut serialization = Vec::new();
            parent.encode_to(&mut serialization);
            assurance.bitfield.encode_to(&mut serialization);
            message.extend_from_slice(&blake2_256(&serialization));
            let validator = &current_validators.validators[assurance.validator_index as usize]; 
            if !assurance.signature.verify_signature(&message, &validator.ed25519) {
                return Err(ErrorCode::BadSignature);
            }
            if assurance.bitfield.len() != AVAIL_BITFIELD_BYTES {
                return Err(ErrorCode::WrongBitfieldLength);
            }
            
            for core in 0..CORES_COUNT {
                let bitfield = assurance.bitfield[core / 8] & (1 << core % 8) != 0;
                if bitfield {
                    // A bit may only be set if the corresponding core has a report pending availability on it
                    if list.assignments[core as usize].is_none() {
                        return Err(ErrorCode::CoreNotEngaged);
                    }
                    core_marks[core as usize] += 1;
                }
            }
        }

        // A work-report is said to become available if and only if there are a clear super-majority of validators
        // who have marked its core as set within the block's assurance extrinsic. We define the sequence of newly
        // available work-reports in the next reported vector.
        let mut reported = Vec::new();
        for core in 0..CORES_COUNT {
            if core_marks[core as usize] >= VALIDATORS_SUPER_MAJORITY {
                if let Some(assignment) = &list.assignments[core as usize] {
                    add_assignment(&AvailabilityAssignment {
                        report: assignment.report.clone(),
                        timeout: post_tau.clone(),
                    });
                    reported.push(assignment.report.clone());
                } 
            } 
        }

        // The Availability Assignments are equivalents except for the removal of items which are now available
        for core in 0..CORES_COUNT {
            if let Some(assignment) = &list.assignments[core as usize] {
                if reported.contains(&assignment.report) {
                    remove_assignment(&(core as CoreIndex));
                }
            }
        }
    
        Ok(OutputDataAssurances { reported } )
    }

}

impl Encode for AssurancesExtrinsic {
    
    fn encode(&self) -> Vec<u8> {

        let mut assurances_blob: Vec<u8> = Vec::new();
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

#[derive(Debug, Clone, PartialEq)]
pub struct OutputDataAssurances {
    pub reported: Vec<WorkReport>
}

impl Encode for OutputDataAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(std::mem::size_of::<WorkReport>() * self.reported.len());
        
        encode_unsigned(self.reported.len()).encode_to(&mut output);
        
        for report in &self.reported {
            report.encode_to(&mut output);
        }

        return output;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputDataAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(OutputDataAssurances {
            reported: {
                let len = decode_unsigned(reader)?;
                let mut reported = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    reported.push(WorkReport::decode(reader)?);
                }
                reported
            }
        })
    }
}
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ErrorCode {
    BadAttestationParent = 0,
    BadValidatorIndex = 1,
    CoreNotEngaged = 2,
    BadSignature = 3,
    NotSortedOrUniqueAssurers = 4,
    TooManyAssurances = 5,
    WrongBitfieldLength = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputAssurances {
    Ok(OutputDataAssurances),
    Err(ErrorCode)
}

impl Encode for OutputAssurances {

    fn encode(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity(std::mem::size_of::<OutputAssurances>());
        match self {
            OutputAssurances::Ok(data) => {
                output.push(0);
                data.encode_to(&mut output);
            },
            OutputAssurances::Err(code) => {
                output.push(1);
                output.push(*code as u8);
            }
        }
        return output;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        let result = reader.read_byte()?;
        match result {
            0 => Ok(OutputAssurances::Ok(OutputDataAssurances::decode(reader)?)),
            1 => {
                let code = reader.read_byte()?;
                let error_code = match code {
                    0 => ErrorCode::BadAttestationParent,
                    1 => ErrorCode::BadValidatorIndex,
                    2 => ErrorCode::CoreNotEngaged,
                    3 => ErrorCode::BadSignature,
                    4 => ErrorCode::NotSortedOrUniqueAssurers,
                    5 => ErrorCode::TooManyAssurances,
                    6 => ErrorCode::WrongBitfieldLength,
                    _ => return Err(ReadError::InvalidData),
                };
                Ok(OutputAssurances::Err(error_code))
            },
            _ => Err(ReadError::InvalidData)
        }
    }
}
