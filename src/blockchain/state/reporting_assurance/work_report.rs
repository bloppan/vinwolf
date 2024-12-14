use sp_core::blake2_256;
use crate::blockchain::state::entropy::get_entropy_state;
use crate::codec::safrole::ValidatorsData;
use crate::codec::Encode;
use crate::utils::common::Verify;
use crate::types::{Ed25519Public, TimeSlot, CoreIndex, Gas};
use crate::constants::{EPOCH_LENGTH, ROTATION_PERIOD, WORK_REPORT_TIMEOUT, WORK_REPORT_GAS_LIMIT};
use crate::codec::work_report::{WorkReport, ReportedPackage, OutputData, OutputWorkReport, AuthPool, AuthPools, ErrorCode};
use crate::codec::refine_context::RefineContext;
use crate::codec::disputes_extrinsic::AvailabilityAssignment;
use crate::blockchain::block::extrinsic::guarantees::ValidatorSignature;
use crate::blockchain::state::validators::{get_validators_state, ValidatorSet};
use crate::blockchain::state::authorization::{get_authpool_state, set_authpool_state};
use crate::blockchain::state::recent_history::get_history_state;
use crate::blockchain::state::time::get_time_state;
use crate::blockchain::state::services::get_services_state;
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

        match self.gas_meets_requirements() {
            Err(error) => return Err(error),
            Ok(_) => {},
        }

        let OutputData {
            reported: new_reported,
            reporters: new_reporters,
        } = self.try_place(post_tau, guarantee_slot, validators_signatures)?;

        return Ok(OutputData{reported: new_reported, reporters: new_reporters});
    }

    fn gas_meets_requirements(&self) -> Result<bool, ErrorCode> {
        let list = get_services_state();
        let mut total_accumulation_gas: Gas = 0;
        // We require that the gas allotted for accumulation of each work item in each work-report respects its service's
        // minimum gas requirements
        'next_result: for result in &self.results {
            for service in &list.services {
                if service.id == result.service {
                    if result.gas >= service.info.balance {
                        return Err(ErrorCode::ServiceItemGasTooLow);
                    }
                    total_accumulation_gas += result.gas;
                    continue 'next_result;
                }
            }
            return Err(ErrorCode::BadServiceId);
        }

        // We also require that all work-reports total allotted accumulation gas is no greater than the WORK_REPORT_GAS_LIMIT
        if total_accumulation_gas > WORK_REPORT_GAS_LIMIT {
            return Err(ErrorCode::WorkReportGasTooHigh);
        }

        return Ok(true);
    }

    fn validate_authorization(&self) -> Result<bool, ErrorCode> {

        let authorization = get_authpool_state();
        // A report is valid only if the authorizer hash is present in the authorizer pool of the core on which the
        // work is reported
        if let Some(_) = authorization.auth_pools.iter()
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
        if availability.assignments[self.core_index as usize].is_none() || *post_tau >= self.context.lookup_anchor_slot + WORK_REPORT_TIMEOUT {

            let chain_entropy = get_entropy_state();
            let current_validators = get_validators_state(ValidatorSet::Current);
            let prev_validators = get_validators_state(ValidatorSet::Previous);

            // Each core has three validators uniquely assigned to guarantee work-reports for it. This is ensured with 
            // VALIDATORS_COUNT and CORES_COUNT, since V/C = 3. The core index is assigned to each of the validators, 
            // and the validator's Ed25519 public keys are denoted as 'guarantors_assignments'.
            // We determine the core to which any given validator is assigned through a shuffle using epochal entropy 
            // and a periodic rotation to help guard the security and liveness of the network. We use η2 (entropy_index 2) 
            // for the epochal entropy rather than η1 to avoid the possibility of fork-magnification where uncertainty 
            // about chain state at the end of an epoch could give rise to two established forks before it naturally resolves.
            let (validators_data, guarantors_assignments) = if *post_tau / ROTATION_PERIOD == guarantee_slot / ROTATION_PERIOD {
                let assignments = guarantor_assignments(&permute(&chain_entropy[2], *post_tau), &mut current_validators.clone());
                (current_validators, assignments)
            } else {
                // We also define the previous 'guarantors_assigments' as it would have been under the previous rotation
                let epoch_diff = (*post_tau - ROTATION_PERIOD) / EPOCH_LENGTH as u32 == *post_tau / EPOCH_LENGTH as u32;
                let entropy_index = if epoch_diff { 2 } else { 3 };
                let validators = if epoch_diff { current_validators } else { prev_validators };
                let assignments = guarantor_assignments(&permute(&chain_entropy[entropy_index], *post_tau), &mut validators.clone());
                (validators, assignments)
            };

            let guarantors_hashmap: std::collections::HashMap<Ed25519Public, CoreIndex> = guarantors_assignments
                .iter()
                .map(|(core_index, public_key)| (*public_key, *core_index))
                .collect();

            // The signature must be one whose public key is that of the validator identified in the credential, and whose
            // message is the serialization of the hash of the work-report.
            let mut message = Vec::from(b"jam_guarantee");
            message.extend_from_slice(&blake2_256(&self.encode()));

            for credential in credentials {
                let validator = &validators_data.validators[credential.validator_index as usize];
                if !credential.signature.verify(&message, &validator.ed25519) {
                    return Err(ErrorCode::BadSignature);
                }
                // The signing validators must be assigned to the core in question in either this block if the timeslot for the
                // guarantee is in the same rotation as this block's timeslot, or in the most recent previous set of assigmments.
                match guarantors_hashmap.get(&validator.ed25519) {
                    Some(&core_index) if core_index == self.core_index => {},
                    Some(_) => return Err(ErrorCode::BadCoreIndex),
                    None => return Err(ErrorCode::GuarantorNotFound),
                }
                if !(ROTATION_PERIOD * ((*post_tau / ROTATION_PERIOD) - 1) <= guarantee_slot && guarantee_slot <= *post_tau) {
                    return Err(ErrorCode::TooOldGuarantee);
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
            add_assignment(&assignment);
        }
        return Ok(OutputData{reported: reported, reporters: reporters});
    }
}