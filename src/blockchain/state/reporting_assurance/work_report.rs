use frame_support::sp_runtime::offchain::storage_lock::Time;

use crate::types::{Ed25519Public, TimeSlot};
use crate::constants::{WORK_REPORT_TIMEOUT, ROTATION_PERIOD};
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


impl WorkReport {

    pub fn validate_authorization(&self) -> Result<bool, ErrorCode> {

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

    pub fn is_recent(&self) -> Result<bool, ErrorCode> {
        
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


    pub fn try_place(&self, slot: TimeSlot, signatures: &[ValidatorSignature]) -> Result<OutputData, ErrorCode> {

        let mut reported: Vec<ReportedPackage> = Vec::new();
        let mut reporters: Vec<Ed25519Public> = Vec::new();

        let curr_validators = get_validators_state(ValidatorSet::Current);
        let prev_validators = get_validators_state(ValidatorSet::Previous);
        //let mut authorization = get_authpool_state();
        let tau = get_time_state();
        let availability = get_reporting_assurance_state();
        // No reports may be placed on cores with a report pending availability on it unless it has timed out.
        // It has timed out, WORK_REPORT_TIMEOUT = 5 slots must have elapsed after de report was made
        if availability.assignments[self.core_index as usize].is_none() || tau > slot + WORK_REPORT_TIMEOUT {
                    
            let assignment = AvailabilityAssignment {
                report: self.clone(),
                timeout: tau.clone(),
            };

            // Update the reporting assurance state
            add_assignment(&assignment);

            // The signing validators must be assigned to the core in either this block if the timeslot for the guarantee 
            // is in the same rotation as this block’s timeslot, or in the most recent previous set of assignments
            let validators = if slot / ROTATION_PERIOD == tau / ROTATION_PERIOD {
                &curr_validators.validators
            } else {
                &prev_validators.validators
            };
            
            for signature in signatures {
                reporters.push(validators[signature.validator_index as usize].ed25519);
            }
            reporters.sort();
            reported.push(ReportedPackage{
                work_package_hash: self.package_spec.hash, 
                segment_tree_root: self.package_spec.exports_root
            });
        }
        return Ok(OutputData{reported: reported, reporters: reporters});
    }
}