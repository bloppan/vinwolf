use sp_core::blake2_256;

use crate::types::{Entropy, Ed25519Public, TimeSlot, CoreIndex, Gas};
use crate::constants::{
    EPOCH_LENGTH, ROTATION_PERIOD, WORK_REPORT_TIMEOUT, WORK_REPORT_GAS_LIMIT, CORES_COUNT, 
    VALIDATORS_COUNT, MAX_AGE_LOOKUP_ANCHOR};

use crate::blockchain::block::extrinsic::disputes::AvailabilityAssignment;
use crate::blockchain::block::extrinsic::guarantees::ValidatorSignature;
use crate::blockchain::state::safrole::codec::ValidatorsData;
use crate::blockchain::state::entropy::get_entropy_state;
use crate::blockchain::state::disputes::get_disputes_state;
use crate::blockchain::state::validators::{get_validators_state, ValidatorSet};
use crate::blockchain::state::authorization::get_authpool_state;
use crate::blockchain::state::recent_history::{self, get_history_state};
use crate::blockchain::state::recent_history::codec::ReportedWorkPackage;
use crate::blockchain::state::services::get_services_state;
use crate::blockchain::state::reporting_assurance::{get_reporting_assurance_staging_state, add_assignment};
use crate::utils::trie::mmr_super_peak;
use crate::utils::shuffle::shuffle;
use crate::utils::codec::Encode;
use crate::utils::common::{VerifySignature, set_offenders_null};
use crate::utils::codec::work_report::{WorkReport, ReportedPackage, OutputData, ErrorCode};


impl WorkReport {

    pub fn process(
        &self, 
        post_tau: &TimeSlot, 
        guarantee_slot: TimeSlot, 
        validators_signatures: &[ValidatorSignature]) 
    -> Result<OutputData, ErrorCode> {

        if !self.validate_authorization()? {
            return Err(ErrorCode::NoAuthorization);
        }

        if !self.is_recent()? {
            return Err(ErrorCode::AnchorNotRecent);
        }

        if let Err(error) = self.results_meets_requirements() {
            return Err(error);
        }

        // We require that each lookup-anchor block be within the last MAX_AGE_LOOKUP_ANCHOR timeslots
        if *post_tau > self.context.lookup_anchor_slot + MAX_AGE_LOOKUP_ANCHOR {
            return Err(ErrorCode::BadLookupAnchorSlot);
        }

        // TODO 11.36 
          
        let OutputData {
            reported: new_reported,
            reporters: new_reporters,
        } = self.try_place(post_tau, guarantee_slot, validators_signatures)?;

        return Ok(OutputData{reported: new_reported, reporters: new_reporters});
    }

    fn results_meets_requirements(&self) -> Result<bool, ErrorCode> {
        let list = get_services_state();
        let mut total_accumulation_gas: Gas = 0;
        
        let service_map: std::collections::HashMap<_, _> = list.services.iter().map(|s| (s.id, s)).collect();

        for result in &self.results {
            if let Some(service) = service_map.get(&result.service) {
                // We require that all work results within the extrinsic predicted the correct code hash for their 
                // corresponding service
                if result.code_hash != service.info.code_hash {
                    return Err(ErrorCode::BadCodeHash);
                }
                // We require that the gas allotted for accumulation of each work item in each work-report respects 
                // its service's minimum gas requirements
                if result.gas < service.info.min_item_gas {
                    return Err(ErrorCode::ServiceItemGasTooLow);
                }
                total_accumulation_gas += result.gas;
            } else {
                return Err(ErrorCode::BadServiceId);
            }
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

        return Err(ErrorCode::CoreUnauthorized);
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

    fn try_place(
        &self, 
        post_tau: &TimeSlot, 
        guarantee_slot: TimeSlot, 
        credentials: &[ValidatorSignature]) 
    -> Result<OutputData, ErrorCode> {

        let mut reported: Vec<ReportedPackage> = Vec::new();
        let mut reporters: Vec<Ed25519Public> = Vec::new();

        let availability = get_reporting_assurance_staging_state();
        // No reports may be placed on cores with a report pending availability on it 
        if availability.assignments[self.core_index as usize].is_none() {

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
                if credential.validator_index as usize >= VALIDATORS_COUNT {
                    return Err(ErrorCode::BadValidatorIndex);
                }
                let validator = &validators_data.validators[credential.validator_index as usize];
                if !credential.signature.verify_signature(&message, &validator.ed25519) {
                    return Err(ErrorCode::BadSignature);
                }
                // The signing validators must be assigned to the core in question in either this block if the timeslot for the
                // guarantee is in the same rotation as this block's timeslot, or in the most recent previous set of assigmments.
                match guarantors_hashmap.get(&validator.ed25519) {
                    Some(&core_index) if core_index == self.core_index => {},
                    Some(_) => return Err(ErrorCode::WrongAssignment),
                    None => return Err(ErrorCode::GuarantorNotFound),
                }
                if !(ROTATION_PERIOD * ((*post_tau / ROTATION_PERIOD) - 1) <= guarantee_slot && guarantee_slot <= *post_tau) {
                    return Err(ErrorCode::FutureReportSlot);
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
            return Ok(OutputData{reported: reported, reporters: reporters});
        } 
        
        return Err(ErrorCode::CoreEngaged);
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
    validators_set: &mut ValidatorsData) 
-> Box<[(CoreIndex, Ed25519Public); VALIDATORS_COUNT]> {

    let mut guarantor_assignments: Box<[(CoreIndex, Ed25519Public); VALIDATORS_COUNT]> = Box::new([(0, Ed25519Public::default()); VALIDATORS_COUNT]);

    set_offenders_null(validators_set, &get_disputes_state().offenders);

    for i in 0..VALIDATORS_COUNT {
        guarantor_assignments[i] = (core_assignments[i], validators_set.validators[i].ed25519.clone());
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