/*    
    The guarantees extrinsic is a series of guarantees, at most one for each core, each of which is a tuple of a work-report, 
    a credential and its corresponding timeslot. The core index of each guarantee must be unique and guarantees must be in 
    ascending order of this. They are reports of newly completed workloads whose accuracy is guaranteed by specific validators. 
    A work-package, which comprises several work items, is transformed by validators acting as guarantors into its corresponding 
    workreport, which similarly comprises several work outputs and then presented on-chain within the guarantees extrinsic.
*/
use std::collections::{HashSet, HashMap};
use constants::node::{CORES_COUNT, MAX_DEPENDENCY_ITEMS, MAX_WORK_ITEMS, WORK_REPORT_GAS_LIMIT};
use jam_types::{
    AvailabilityAssignments, CoreIndex, EntropyPool, Hash, OutputDataReports, ProcessError, ReportErrorCode, TimeSlot, ValidatorIndex, 
    ValidatorsData, OpaqueHash, WorkReport, ValidatorSignature, ReadError, Ed25519Signature, ReportGuarantee
};
use codec::{Encode, EncodeSize, Decode, BytesReader};
use codec::generic_codec::{encode_unsigned, decode_unsigned};
use crate::GuaranteesExtrinsic;
use utils::common::is_sorted_and_unique;
use handler::{get_accumulation_history, get_ready_queue, get_recent_history, get_reporting_assurance};

impl Default for GuaranteesExtrinsic {
    fn default() -> Self {
        GuaranteesExtrinsic {
            report_guarantee: Vec::new(),
        }
    }
}

impl GuaranteesExtrinsic {

    pub fn process(
        &self, 
        assurances_state: &mut AvailabilityAssignments, 
        post_tau: &TimeSlot,
        entropy_pool: &EntropyPool,
        prev_validators: &ValidatorsData,
        curr_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        log::debug!("Processing guarantees extrinsic...");
        // At most one guarantee for each core
        if self.report_guarantee.len() > CORES_COUNT {
            log::error!("Too many guarantees: {:?}", self.report_guarantee.len());
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyGuarantees));
        }

        // There must be no duplicate work-package hashes (i.e. two work-reports of the same package).
        let mut packages_hashes = self.report_guarantee.iter()
                                                                        .map(|i| i.report.package_spec.hash)
                                                                        .collect::<Vec<_>>();
        packages_hashes.sort(); 
        if !is_sorted_and_unique(&packages_hashes) {
            log::error!("Duplicate package in guarantees extrinsic");
            return Err(ProcessError::ReportError(ReportErrorCode::DuplicatePackage));
        }
        
        // Therefore, we require the cardinality of all work-packages to be the length of the work-report sequence
        if packages_hashes.len() != self.report_guarantee.len() {
            log::error!("Length not equal in guarantees extrinsic: packages hashes length {:?} != guarantees length {:?}", 
                        packages_hashes.len(), self.report_guarantee.len());
            return Err(ProcessError::ReportError(ReportErrorCode::LengthNotEqual));
        }

        // We limit the sum of the number of items in the segment-root lookup dictionary and the number of prerequisites to MAX_DEPENDENCY_ITEMS
        for guarantee in &self.report_guarantee {
            if guarantee.report.context.prerequisites.len() + guarantee.report.segment_root_lookup.len() > MAX_DEPENDENCY_ITEMS {
                log::error!("Too many dependencies: {:?} > MAX_DEPENDENCY_ITEMS: {:?}", 
                            guarantee.report.context.prerequisites.len() + guarantee.report.segment_root_lookup.len(), MAX_DEPENDENCY_ITEMS);
                return Err(ProcessError::ReportError(ReportErrorCode::TooManyDependencies));
            }
        }

        let mut reported = Vec::new();
        let mut reporters = Vec::new();
        let mut core_index: Vec<CoreIndex> = Vec::new();
        let recent_history = get_recent_history();

        let packages_map: HashMap<Hash, Hash> = self.report_guarantee.iter()
                                                                     .map(|g| (g.report.package_spec.hash, g.report.package_spec.exports_root))
                                                                     .collect();
        
        // We require that the work-package of the report not be the work-package of some other report made in the past.
        // We ensure that the work-package not appear anywhere within our pipeline
        let mut wp_hashes_in_our_pipeline: std::collections::HashSet<OpaqueHash> = HashSet::new();

        let recent_history_map: std::collections::HashMap<_, _> = recent_history.blocks
            .iter()
            .flat_map(|blocks| blocks.reported_wp.iter())
            .map(|report| (report.0, report.1))
            .collect();

        for hash in recent_history_map.iter() {
            wp_hashes_in_our_pipeline.insert(hash.0.clone());
        }

        let acc_queue = get_ready_queue();
        for epoch in acc_queue.queue.iter() {
            for ready_record in epoch.iter() {
                wp_hashes_in_our_pipeline.extend(ready_record.dependencies.clone());
            }
        }
        
        let acc_history = get_accumulation_history();
        for item in acc_history.queue.iter() {
            wp_hashes_in_our_pipeline.extend(item.clone());
        }

        let assurance_state = get_reporting_assurance();
        for item in assurance_state.list.iter() {
            if let Some(assignment) = item {
                wp_hashes_in_our_pipeline.extend(&assignment.report.context.prerequisites.clone());
            }
        }

        for guarantee in &self.report_guarantee {
       
            // The core index of each guarantee must be unique and guarantees must be in ascending order of this
            core_index.push(guarantee.report.core_index);
            if !is_sorted_and_unique(&core_index) {
                log::error!("Out of order guarantee");
                return Err(ProcessError::ReportError(ReportErrorCode::OutOfOrderGuarantee));
            }

            if guarantee.report.core_index > CORES_COUNT as CoreIndex {
                log::error!("Bad core index: {:?}. The total of cores is {:?}", guarantee.report.core_index, CORES_COUNT);
                return Err(ProcessError::ReportError(ReportErrorCode::BadCoreIndex));
            }

            // The credential is a sequence of two or three tuples of a unique validator index and a signature
            if guarantee.signatures.len() < 2 || guarantee.signatures.len() > 3 {
                log::error!("Insufficient guarantees signatures: {:?}", guarantee.signatures.len());
                return Err(ProcessError::ReportError(ReportErrorCode::InsufficientGuarantees));
            }

            // Credentials must be ordered by their validator index
            let validator_indexes: Vec<ValidatorIndex> = guarantee.signatures.iter().map(|i| i.validator_index).collect();
            if !is_sorted_and_unique(&validator_indexes) {
                log::error!("Not sorted or unique guarantors");
                return Err(ProcessError::ReportError(ReportErrorCode::NotSortedOrUniqueGuarantors));
            }

            // We require that the work-package of the report not be the work-package of some other report made in the past.
            // We ensure that the work-package not appear anywhere within our pipeline.
            if wp_hashes_in_our_pipeline.contains(&guarantee.report.package_spec.hash) {
                log::error!("Duplicate package 0x{}", utils::print_hash!(guarantee.report.package_spec.hash));
                return Err(ProcessError::ReportError(ReportErrorCode::DuplicatePackage));
            }
 
            // We require that the prerequisite work-packages, if present, be either in the extrinsic or in our recent history 
            for prerequisite in &guarantee.report.context.prerequisites {
                if !packages_map.contains_key(prerequisite) && !recent_history_map.contains_key(prerequisite) {
                    log::error!("Dependency missing 0x{}", utils::print_hash!(*prerequisite));
                    return Err(ProcessError::ReportError(ReportErrorCode::DependencyMissing));
                }
            }
            
            // We require that any work-packages mentioned in the segment-root lookup, if present, be either in the extrinsic
            // or in our recent history
            for segment in &guarantee.report.segment_root_lookup {
                let segment_root = packages_map.get(&segment.work_package_hash)
                    .or_else(|| recent_history_map.get(&segment.work_package_hash));
                // We require that any segment roots mentioned in the segment-root lookup be verified as correct based on our
                // recent work-package history and the present block
                match segment_root {
                    Some(&value) if value == segment.segment_tree_root => continue,
                    _ => {
                        log::error!("Segment root lookup invalid");
                        return Err(ProcessError::ReportError(ReportErrorCode::SegmentRootLookupInvalid));
                    },
                }
            }

            // Process the work report
            let OutputDataReports {
                reported: new_reported,
                reporters: new_reporters,
            } = work_report::process(&guarantee.report,
                                    assurances_state, 
                                    post_tau, 
                                    guarantee.slot, 
                                    &guarantee.signatures, 
                                    entropy_pool,
                                    prev_validators,
                                    curr_validators)?;
    
            reported.extend(new_reported);
            reporters.extend(new_reporters);
        }
    
        reported.sort_by_key(|report| report.work_package_hash);
        reporters.sort();
        /*reported.sort_by(|a, b| a.work_package_hash.cmp(&b.work_package_hash));
        reporters.sort();*/
        log::debug!("Guarantees extrinsic processed successfully");

        Ok(OutputDataReports { reported, reporters })
    }
    
}

use sp_core::blake2_256;

use jam_types::{
    AvailabilityAssignment, Ed25519Public, Entropy,
    ReportedPackage, WorkResult
};
use constants::node::{ EPOCH_LENGTH, ROTATION_PERIOD, MAX_OUTPUT_BLOB_SIZE, VALIDATORS_COUNT, MAX_AGE_LOOKUP_ANCHOR };
use handler::{ get_auth_pools, get_disputes};
use handler::get_current_block_history;
use handler::add_assignment;
use utils::trie::mmr_super_peak;
use utils::shuffle::shuffle;
use utils::common::{VerifySignature, set_offenders_null};

pub mod work_report {
    use super::*;

    pub fn process(
        work_report: &WorkReport,
        assurances_state: &mut AvailabilityAssignments,
        post_tau: &TimeSlot, 
        guarantee_slot: TimeSlot, 
        validators_signatures: &[ValidatorSignature],
        entropy_pool: &EntropyPool,
        prev_validators: &ValidatorsData,
        curr_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        log::debug!("Processing work report 0x{}", utils::print_hash!(work_report.package_spec.hash));

        let auth_pools = get_auth_pools();
        // A report is valid only if the authorizer hash is present in the authorizer pool of the core on which the
        // work is reported
        if !auth_pools.0[work_report.core_index as usize].contains(&work_report.authorizer_hash) {
            log::error!("Core {:?} unauthorized. Could not found 0x{} auth hash", work_report.core_index, utils::print_hash!(work_report.authorizer_hash));
            return Err(ProcessError::ReportError(ReportErrorCode::CoreUnauthorized));
        }

        // We require that the anchor block be within the last RECENT_HISTORY_SIZE blocks and that its details be correct 
        // by ensuring that it appears within our most recent blocks
        if let Err(error) = is_recent(work_report) {
            return Err(error);
        }

        let mut work_report_size = 0;
        // We require that the work-report's results are valid
        match work_result::process(&work_report.results) {
            Ok(results_size) => { work_report_size += results_size },
            Err(e) => { return Err(e) },
        }
        // In order to ensure fair use of a block’s extrinsic space, work-reports are limited in the maximum total size of 
        // the successful output blobs together with the authorizer output blob, effectively limiting their overall size
        if work_report_size + work_report.auth_output.len() > MAX_OUTPUT_BLOB_SIZE {
            log::error!("Work report too big: {:?}. The max output blob size is {:?}", work_report_size + work_report.auth_output.len(), MAX_OUTPUT_BLOB_SIZE);
            return Err(ProcessError::ReportError(ReportErrorCode::WorkReportTooBig));
        }

        // We require that each lookup-anchor block be within the last MAX_AGE_LOOKUP_ANCHOR timeslots
        if *post_tau > work_report.context.lookup_anchor_slot + MAX_AGE_LOOKUP_ANCHOR {
            log::error!("Bad lookup anchor slot. Current slot {:?} > lookup anchor slot + MAX AGE LOOKUP ANCHOR {:?}", 
                        *post_tau, work_report.context.lookup_anchor_slot + MAX_AGE_LOOKUP_ANCHOR);
            return Err(ProcessError::ReportError(ReportErrorCode::BadLookupAnchorSlot));
        }

        // TODO 11.35
        
        // We require that the prerequisite work-packages, if present, and any work-packages mentioned in the segment-root lookup,
        // be either in the extrinsic or in our recent history.
        /*let mut wp_hashes_in_our_pipeline: HashSet<OpaqueHash> = HashSet::new();

        let acc_queue = get_ready_queue();
        for epoch in acc_queue.queue.iter() {
            for ready_record in epoch.iter() {
                wp_hashes_in_our_pipeline.extend(ready_record.dependencies.clone());
            }
        }
        
        let assurance_state = get_reporting_assurance();
        for item in assurance_state.list.iter() {
            if let Some(assignment) = item {
                wp_hashes_in_our_pipeline.extend(&assignment.report.context.prerequisites);
            }
        }*/

        let OutputDataReports {
            reported: new_reported,
            reporters: new_reporters,
        } = work_report::try_place(work_report,
                    assurances_state, 
                    post_tau, 
                    guarantee_slot, 
                    validators_signatures, 
                    entropy_pool,
                    prev_validators,
                    curr_validators)?;

        log::debug!("Work report 0x{} processed successfully", utils::print_hash!(work_report.package_spec.hash));
        return Ok(OutputDataReports{reported: new_reported, reporters: new_reporters});
    }

    fn is_recent(work_report: &WorkReport) -> Result<bool, ProcessError> {
        
        let block_history = get_current_block_history().lock().unwrap().clone();

        for block in &block_history.blocks {
            if block.header_hash == work_report.context.anchor {
                if block.state_root != work_report.context.state_root {
                    log::error!("Bad state root. Block state root 0x{} != Context state root 0x{}", 
                                utils::print_hash!(block.state_root), utils::print_hash!(work_report.context.state_root));
                    return Err(ProcessError::ReportError(ReportErrorCode::BadStateRoot));
                }

                if mmr_super_peak(&block.mmr) != work_report.context.beefy_root {
                    log::error!("Bad beefy MMR Root");
                    return Err(ProcessError::ReportError(ReportErrorCode::BadBeefyMmrRoot));
                }
        
                log::debug!("The block anchor is recent");
                return Ok(true);
            }
        }

        log::error!("Anchor not recent");
        Err(ProcessError::ReportError(ReportErrorCode::AnchorNotRecent))
    }

    fn try_place(work_report: &WorkReport,
                 assurances_state: &mut AvailabilityAssignments,
                 post_tau: &TimeSlot, 
                 guarantee_slot: TimeSlot, 
                 credentials: &[ValidatorSignature],
                 entropy_pool: &EntropyPool,
                 prev_validators: &ValidatorsData,
                 current_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        log::debug!("Try place work report 0x{}", utils::print_hash!(work_report.package_spec.hash));
        
        let mut reported: Vec<ReportedPackage> = Vec::new();
        let mut reporters: Vec<Ed25519Public> = Vec::new();

        // No reports may be placed on cores with a report pending availability on it 
        if assurances_state.list[work_report.core_index as usize].is_none() {
            // Each core has three validators uniquely assigned to guarantee work-reports for it. This is ensured with 
            // VALIDATORS_COUNT and CORES_COUNT, since V/C = 3. The core index is assigned to each of the validators, 
            // and the validator's Ed25519 public keys are denoted as 'assignments'.
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
            let message = [&b"jam_guarantee"[..], &blake2_256(&work_report.encode())].concat();

            for credential in credentials {
                if credential.validator_index as usize >= VALIDATORS_COUNT {
                    log::error!("Bad validator index: {:?}", credential.validator_index);
                    return Err(ProcessError::ReportError(ReportErrorCode::BadValidatorIndex));
                }
                let validator = &validators_data.list[credential.validator_index as usize];

                if !credential.signature.verify_signature(&message, &validator.ed25519) {
                    log::error!("Bad signature");
                    return Err(ProcessError::ReportError(ReportErrorCode::BadSignature));
                }
                if ROTATION_PERIOD * ((*post_tau / ROTATION_PERIOD).saturating_sub(1)) > guarantee_slot {
                    log::error!("Report epoch before last");
                    return Err(ProcessError::ReportError(ReportErrorCode::ReportEpochBeforeLast));
                }
                if guarantee_slot > *post_tau {
                    log::error!("Future report slot: {:?}. The current block slot is {:?}", guarantee_slot, *post_tau);
                    return Err(ProcessError::ReportError(ReportErrorCode::FutureReportSlot));
                }
                // The signing validators must be assigned to the core in question in either this block if the timeslot for the
                // guarantee is in the same rotation as this block's timeslot, or in the most recent previous set of assigmments.
                if let Some(&core_index) = assignments.get(&validator.ed25519) {
                    if core_index != work_report.core_index {
                        log::error!("Wrong assignment {:?} != {:?}", core_index, work_report.core_index);
                        return Err(ProcessError::ReportError(ReportErrorCode::WrongAssignment));
                    }
                } else {
                    log::error!("Guarantor not found");
                    return Err(ProcessError::ReportError(ReportErrorCode::GuarantorNotFound));
                }
                // We note that the Ed25519 key of each validator whose signature is in a credential is placed in the reporters set.
                // This is utilized by the validator activity statistics book-keeping system.
                reporters.push(validator.ed25519);
            }

            reporters.sort();
            reported.push(ReportedPackage{
                work_package_hash: work_report.package_spec.hash, 
                segment_tree_root: work_report.package_spec.exports_root
            });

            // In the case an entry is replaced, the new value includes the present time 'post_tau' allowing for the value to be 
            // replaced without respect to its availability once sufficient time has elapsed.
            let assignment = AvailabilityAssignment {
                report: work_report.clone(),
                timeout: *post_tau,
            };

            // Update the reporting assurance state
            add_assignment(&assignment, assurances_state);

            log::debug!("The work report was placed successfully");
            return Ok(OutputDataReports{reported: reported, reporters: reporters});
        } 
        
        log::error!("Core {:?} engaged", work_report.core_index);
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

use jam_types::{Gas};
use handler::get_service_accounts;

mod work_result {

    use super::*;

    pub fn process(results: &[WorkResult]) -> Result<usize, ProcessError> {

        log::debug!("Processing work results");

        if results.len() < 1 {
            log::error!("No results");
            return Err(ProcessError::ReportError(ReportErrorCode::NoResults));
        }

        if results.len() > MAX_WORK_ITEMS {
            log::error!("Too many results: {:?}", results.len());
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyResults));
        }

        let services = get_service_accounts();
        let mut total_accumulation_gas: Gas = 0;
        
        //let service_map: std::collections::HashMap<_, _> = services.0.iter().map(|s| (s.id, s)).collect();
        let mut results_size = 0;

        for result in results.iter() {
            if let Some(service) = services.get(&result.service) {
                // We require that all work results within the extrinsic predicted the correct code hash for their 
                // corresponding service
                if result.code_hash != service.code_hash {
                    log::error!("Bad code hash 0x{} != 0x{}", utils::print_hash!(result.code_hash), utils::print_hash!(service.code_hash));
                    return Err(ProcessError::ReportError(ReportErrorCode::BadCodeHash));
                }
                // We require that the gas allotted for accumulation of each work item in each work-report respects 
                // its service's minimum gas requirements
                // TODO revisar esto a ver si en realidad es este gas
                if result.gas < service.acc_min_gas {
                    log::error!("Service item gas too low: {:?}. The min gas required is {:?}", result.gas, service.acc_min_gas);
                    return Err(ProcessError::ReportError(ReportErrorCode::ServiceItemGasTooLow));
                }
                total_accumulation_gas += result.gas;

                if result.result[0] == 0 {
                    results_size += result.result.len() - 1;
                }
            } else {
                log::error!("Bad service id: {:?}", result.service);
                return Err(ProcessError::ReportError(ReportErrorCode::BadServiceId));
            }
        }

        // We also require that all work-reports total allotted accumulation gas is no greater than the WORK_REPORT_GAS_LIMIT
        if total_accumulation_gas > WORK_REPORT_GAS_LIMIT {
            log::error!("Work report gas too high: {:?}. The work report gas limit is {:?}", total_accumulation_gas, WORK_REPORT_GAS_LIMIT);
            return Err(ProcessError::ReportError(ReportErrorCode::WorkReportGasTooHigh));
        }

        log::debug!("Work results processed successfully");
        return Ok(results_size);
    }
}

impl Encode for GuaranteesExtrinsic {

    fn encode(&self) -> Vec<u8> {

        let mut guarantees_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>() * self.report_guarantee.len());
        encode_unsigned(self.report_guarantee.len()).encode_to(&mut guarantees_blob);

        for guarantee in &self.report_guarantee {

            guarantee.report.encode_to(&mut guarantees_blob);
            guarantee.slot.encode_size(4).encode_to(&mut guarantees_blob);
            encode_unsigned(guarantee.signatures.len()).encode_to(&mut guarantees_blob);

            for signature in &guarantee.signatures {
                signature.validator_index.encode_to(&mut guarantees_blob);
                signature.signature.encode_to(&mut guarantees_blob);
            }
        }

        return guarantees_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for GuaranteesExtrinsic {

    fn decode(guarantees_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let num_guarantees = decode_unsigned(guarantees_blob)?;
        let mut report_guarantee: Vec<ReportGuarantee> = Vec::with_capacity(num_guarantees);
        for _ in 0..num_guarantees {
            let report = WorkReport::decode(guarantees_blob)?;
            let slot = TimeSlot::decode(guarantees_blob)?;
            let num_signatures = decode_unsigned(guarantees_blob)?;
            let mut signatures: Vec<ValidatorSignature> = Vec::with_capacity(num_signatures);

            for _ in 0..num_signatures {
                let validator_index = ValidatorIndex::decode(guarantees_blob)?;
                let signature = Ed25519Signature::decode(guarantees_blob)?;
                signatures.push(ValidatorSignature{validator_index, signature});
            }

            report_guarantee.push(ReportGuarantee{report, slot, signatures});

        }
        Ok(GuaranteesExtrinsic{ report_guarantee })
    }
}