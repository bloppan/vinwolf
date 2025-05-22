use sp_core::blake2_256;
use std::collections::HashMap;

use crate::types::{
    AvailabilityAssignment, AvailabilityAssignments, CoreIndex, Ed25519Public, Entropy, EntropyPool, Hash, OutputDataReports, ReportErrorCode, 
    ReportedPackage, TimeSlot, ValidatorSignature, ValidatorsData, WorkReport, WorkResult
};
use crate::constants::{ EPOCH_LENGTH, ROTATION_PERIOD, MAX_OUTPUT_BLOB_SIZE, CORES_COUNT, VALIDATORS_COUNT, MAX_AGE_LOOKUP_ANCHOR };
use crate::blockchain::state::{ ProcessError, get_auth_pools, get_recent_history, get_disputes };
use crate::blockchain::state::reporting_assurance::add_assignment;
use crate::utils::trie::mmr_super_peak;
use crate::utils::shuffle::shuffle;
use crate::utils::codec::Encode;
use crate::utils::common::{VerifySignature, set_offenders_null};

impl WorkReport {

    pub fn process(
        &self,
        assurances_state: &mut AvailabilityAssignments,
        post_tau: &TimeSlot, 
        guarantee_slot: TimeSlot, 
        validators_signatures: &[ValidatorSignature],
        entropy_pool: &EntropyPool,
        prev_validators: &ValidatorsData,
        curr_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        let auth_pools = get_auth_pools();
        // A report is valid only if the authorizer hash is present in the authorizer pool of the core on which the
        // work is reported
        if !auth_pools.0[self.core_index as usize].contains(&self.authorizer_hash) {
            return Err(ProcessError::ReportError(ReportErrorCode::CoreUnauthorized));
        }

        // We require that the anchor block be within the last RECENT_HISTORY_SIZE blocks and that its details be correct 
        // by ensuring that it appears within our most recent blocks
        if let Err(error) = self.is_recent() {
            return Err(error);
        }

        let mut work_report_size = 0;
        // We require that the work-report's results are valid
        match WorkResult::process(&self.results) {
            Ok(results_size) => { work_report_size += results_size },
            Err(e) => { return Err(e) },
        }
        // In order to ensure fair use of a block’s extrinsic space, work-reports are limited in the maximum total size of 
        // the successful output blobs together with the authorizer output blob, effectively limiting their overall size
        if work_report_size + self.auth_output.len() > MAX_OUTPUT_BLOB_SIZE {
            return Err(ProcessError::ReportError(ReportErrorCode::WorkReportTooBig));
        }

        // We require that each lookup-anchor block be within the last MAX_AGE_LOOKUP_ANCHOR timeslots
        if *post_tau > self.context.lookup_anchor_slot + MAX_AGE_LOOKUP_ANCHOR {
            return Err(ProcessError::ReportError(ReportErrorCode::BadLookupAnchorSlot));
        }

        // TODO 11.36 
        // TODO 11.37

        let OutputDataReports {
            reported: new_reported,
            reporters: new_reporters,
        } = self.try_place(
                    assurances_state, 
                    post_tau, 
                    guarantee_slot, 
                    validators_signatures, 
                    entropy_pool,
                    prev_validators,
                    curr_validators)?;

        return Ok(OutputDataReports{reported: new_reported, reporters: new_reporters});
    }

    fn is_recent(&self) -> Result<bool, ProcessError> {
        
        let block_history = get_recent_history();

        for block in &block_history.blocks {
            if block.header_hash == self.context.anchor {
                if block.state_root != self.context.state_root {
                    return Err(ProcessError::ReportError(ReportErrorCode::BadStateRoot));
                }

                if mmr_super_peak(&block.mmr) != self.context.beefy_root {
                    return Err(ProcessError::ReportError(ReportErrorCode::BadBeefyMmrRoot));
                }

                return Ok(true);
            }
        }

        Err(ProcessError::ReportError(ReportErrorCode::AnchorNotRecent))
    }

    fn try_place(&self,
                 assurances_state: &mut AvailabilityAssignments,
                 post_tau: &TimeSlot, 
                 guarantee_slot: TimeSlot, 
                 credentials: &[ValidatorSignature],
                 entropy_pool: &EntropyPool,
                 prev_validators: &ValidatorsData,
                 current_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        let mut reported: Vec<ReportedPackage> = Vec::new();
        let mut reporters: Vec<Ed25519Public> = Vec::new();

        // No reports may be placed on cores with a report pending availability on it 
        if assurances_state.list[self.core_index as usize].is_none() {
            // Each core has three validators uniquely assigned to guarantee work-reports for it. This is ensured with 
            // VALIDATORS_COUNT and CORES_COUNT, since V/C = 3. The core index is assigned to each of the validators, 
            // and the validator's Ed25519 public keys are denoted as 'guarantors_assignments'.
            // We determine the core to which any given validator is assigned through a shuffle using epochal entropy 
            // and a periodic rotation to help guard the security and liveness of the network. We use η2 (entropy_index 2) 
            // for the epochal entropy rather than η1 to avoid the possibility of fork-magnification where uncertainty 
            // about chain state at the end of an epoch could give rise to two established forks before it naturally resolves.
            let (validators_data, assignments) = if *post_tau / ROTATION_PERIOD == guarantee_slot / ROTATION_PERIOD {
                let mut validators = current_validators.clone();
                let assignments = guarantor_assignments(&permute(&entropy_pool.buf[2], *post_tau), &mut validators);
                (validators, assignments)
            } else {
                // We also define the previous 'guarantors_assigments' as it would have been under the previous rotation
                let epoch_diff = (*post_tau - ROTATION_PERIOD) / EPOCH_LENGTH as u32 == *post_tau / EPOCH_LENGTH as u32;
                let entropy_index = if epoch_diff { 2 } else { 3 };
                let mut validators = if epoch_diff { current_validators.clone() } else { prev_validators.clone() };
                let assignments = guarantor_assignments(&permute(&entropy_pool.buf[entropy_index], *post_tau - ROTATION_PERIOD), &mut validators);
                (validators, assignments)
            };
            
            // The signature must be one whose public key is that of the validator identified in the credential, and whose
            // message is the serialization of the hash of the work-report.
            let message = [&b"jam_guarantee"[..], &blake2_256(&self.encode())].concat();

            for credential in credentials {
                if credential.validator_index as usize >= VALIDATORS_COUNT {
                    return Err(ProcessError::ReportError(ReportErrorCode::BadValidatorIndex));
                }
                let validator = &validators_data.list[credential.validator_index as usize];
                if !credential.signature.verify_signature(&message, &validator.ed25519) {
                    return Err(ProcessError::ReportError(ReportErrorCode::BadSignature));
                }
                if ROTATION_PERIOD * ((*post_tau / ROTATION_PERIOD) - 1) > guarantee_slot {
                    return Err(ProcessError::ReportError(ReportErrorCode::ReportEpochBeforeLast));
                }
                if guarantee_slot > *post_tau {
                    return Err(ProcessError::ReportError(ReportErrorCode::FutureReportSlot));
                }
                // The signing validators must be assigned to the core in question in either this block if the timeslot for the
                // guarantee is in the same rotation as this block's timeslot, or in the most recent previous set of assigmments.
                if let Some(&core_index) = assignments.get(&validator.ed25519) {
                    if core_index != self.core_index {
                        return Err(ProcessError::ReportError(ReportErrorCode::WrongAssignment));
                    }
                } else {
                    return Err(ProcessError::ReportError(ReportErrorCode::GuarantorNotFound));
                }
                // We note that the Ed25519 key of each validator whose signature is in a credential is placed in the reporters set.
                // This is utilized by the validator activity statistics book-keeping system.
                reporters.push(validator.ed25519);
            }

            reporters.sort();
            reported.push(ReportedPackage{
                work_package_hash: self.package_spec.hash, 
                segment_tree_root: self.package_spec.exports_root
            });

            // In the case an entry is replaced, the new value includes the present time 'post_tau' allowing for the value to be 
            // replaced without respect to its availability once sufficient time has elapsed.
            let assignment = AvailabilityAssignment {
                report: self.clone(),
                timeout: *post_tau,
            };

            // Update the reporting assurance state
            add_assignment(&assignment, assurances_state);
            return Ok(OutputDataReports{reported: reported, reporters: reporters});
        } 
        
        return Err(ProcessError::ReportError(ReportErrorCode::CoreEngaged));
    }
}

fn rotation(c: &[u16], n: u16) -> Vec<u16> {

    let mut result: Vec<u16> = Vec::with_capacity(c.len());

    for x in c {
        result.push((x + n) % CORES_COUNT as u16);
    }

    return result;
}

fn permute(entropy: &Entropy, t: TimeSlot) -> Vec<u16> {

    let mut items: Vec<u16> = Vec::with_capacity(VALIDATORS_COUNT);

    for i in 0..VALIDATORS_COUNT {
        items.push(((CORES_COUNT * i) / VALIDATORS_COUNT) as u16);
    }

    let res_shuffle = shuffle(&items, entropy);
    let n = ((t as u32 % EPOCH_LENGTH as u32) as u16) / ROTATION_PERIOD as u16;
    rotation(&res_shuffle, n)
}

fn guarantor_assignments(
    core_assignments: &[u16], 
    validators_set: &mut ValidatorsData
) -> HashMap<Ed25519Public, CoreIndex> {

    let mut guarantor_assignments: HashMap<Ed25519Public, CoreIndex> = HashMap::new();

    set_offenders_null(validators_set, &get_disputes().offenders);

    for i in 0..VALIDATORS_COUNT {
        guarantor_assignments.insert(validators_set.list[i].ed25519.clone(), core_assignments[i]);
    }

    return guarantor_assignments;
}   

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn rotation_test() {

        let c: Vec<u16> = vec![0, 1, 2, 3, 4, 5];
        let n = 5;

        assert_eq!(vec![1, 0, 1, 0, 1, 0], rotation(&c, n));
    }
}