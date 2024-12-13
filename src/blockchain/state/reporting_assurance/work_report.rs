use sp_core::blake2_256;
use crate::blockchain::state::entropy::get_entropy_state;
use crate::codec::safrole::ValidatorsData;
use crate::codec::Encode;
use crate::utils::common::Verify;
use crate::types::{Ed25519Public, TimeSlot};
use crate::constants::{EPOCH_LENGTH, ROTATION_PERIOD, WORK_REPORT_TIMEOUT};
use crate::codec::work_report::{WorkReport, ReportedPackage, OutputData, OutputWorkReport, AuthPool, AuthPools, ErrorCode};
use crate::codec::refine_context::RefineContext;
use crate::codec::disputes_extrinsic::AvailabilityAssignment;
use crate::blockchain::block::extrinsic::guarantees::ValidatorSignature;
use crate::blockchain::state::validators::{get_validators_state, ValidatorSet};
use crate::blockchain::state::authorization::{get_authpool_state, set_authpool_state};
use crate::blockchain::state::recent_history::get_history_state;
use crate::blockchain::state::time::get_time_state;
use crate::blockchain::state::reporting_assurance::{get_reporting_assurance_state, add_assignment};
use crate::trie::mmr_super_peak;

use super::{guarantor_assignments, permute};


impl WorkReport {

    pub fn process(&self, post_tau: &TimeSlot, guarantee_slot: TimeSlot, validators_signatures: &[ValidatorSignature]) -> Result<OutputData, ErrorCode> {

        if !self.validate_authorization()? {
            return Err(ErrorCode::NoAuthorization);
        }

        if !self.is_recent()? {
            return Err(ErrorCode::AnchorNotRecent);
        }

        let OutputData {
            reported: new_reported,
            reporters: new_reporters,
        } = self.try_place(post_tau, guarantee_slot, validators_signatures)?;

        return Ok(OutputData{reported: new_reported, reporters: new_reporters});
    }

    fn validate_authorization(&self) -> Result<bool, ErrorCode> {

        let mut authorization = get_authpool_state();
        // A report is valid only if the authorizer hash is present in the authorizer pool of the core on which the
        // work is reported
        if let Some(auth_pool) = authorization.auth_pools.iter_mut()
                                                                        .find(|auth| auth.auth_pool
                                                                            .contains(&self.authorizer_hash))
        {
            return Ok(true);
        }

        return Err(ErrorCode::NoAuthorization);
    }

    fn is_recent(&self) -> Result<bool, ErrorCode> {
        
        let block_history = get_history_state();

        for block in &block_history.beta {
            if block.header_hash == self.context.anchor {
                if block.state_root != self.context.state_root {
                    return Err(ErrorCode::BadStateRoot);
                }

                if mmr_super_peak(&block.mmr) != self.context.beefy_root {
                    return Err(ErrorCode::BadBeefyMmrRoot);
                }

                return Ok(true);
            }
        }

        Err(ErrorCode::AnchorNotRecent)
    }

    fn try_place(&self, post_tau: &TimeSlot, guarantee_slot: TimeSlot, credentials: &[ValidatorSignature]) -> Result<OutputData, ErrorCode> {

        let mut reported: Vec<ReportedPackage> = Vec::new();
        let mut reporters: Vec<Ed25519Public> = Vec::new();

        let availability = get_reporting_assurance_state();
        // No reports may be placed on cores with a report pending availability on it unless it has timed out.
        // It has timed out, WORK_REPORT_TIMEOUT = 5 slots must have elapsed after de report was made
        if availability.assignments[self.core_index as usize].is_none() || *post_tau > self.context.lookup_anchor_slot + WORK_REPORT_TIMEOUT {

            let chain_entropy = get_entropy_state();
            // The signing validators must be assigned to the core in either this block if the timeslot for the guarantee 
            // is in the same rotation as this block’s timeslot, or in the most recent previous set of assignments
            let mut current_validators = get_validators_state(ValidatorSet::Current);
            let mut prev_validators = get_validators_state(ValidatorSet::Previous);

            let (validators_data, guarantors_assigments) = if *post_tau / ROTATION_PERIOD == guarantee_slot / ROTATION_PERIOD {
                let guarantors_assignments = guarantor_assignments(&permute(&chain_entropy[2], *post_tau), &mut current_validators);
                (current_validators, guarantors_assignments)
            } else {
                if (*post_tau - ROTATION_PERIOD) / EPOCH_LENGTH as u32 == *post_tau / EPOCH_LENGTH as u32 {
                    let prev_guarantor_assignments = guarantor_assignments(&permute(&chain_entropy[2], *post_tau), &mut current_validators);
                    (current_validators, prev_guarantor_assignments)
                } else {
                    let prev_guarantor_assignments =  guarantor_assignments(&permute(&chain_entropy[3], *post_tau), &mut prev_validators);
                    (prev_validators, prev_guarantor_assignments)
                }
            };
            
            
            // The signature must be one whose public key is that of the validator identified in the credential, and whose 
            // message is the serialization of the hash of the work-report.
            let mut message = Vec::from(b"jam_guarantee");
            message.extend_from_slice(&blake2_256(&self.encode()));

            for i in 0..credentials.len() {
                if !credentials[i].signature.verify(&message, &validators_data.validators[credentials[i].validator_index as usize].ed25519) {
                    return Err(ErrorCode::BadSignature);
                }
                /*if guarantors_assigments[i].0 != self.core_index {
                    return Err(ErrorCode::BadValidatorIndex);
                }*/
                /*if ROTATION_PERIOD * ((*post_tau / ROTATION_PERIOD) - 1) <= guarantee_slot && guarantee_slot <= *post_tau {
                    return Err(ErrorCode::TooOldGuarantee);
                }*/
                reporters.push(validators_data.validators[credentials[i].validator_index as usize].ed25519);
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
            add_assignment(&assignment);
        }
        return Ok(OutputData{reported: reported, reporters: reporters});
    }
}