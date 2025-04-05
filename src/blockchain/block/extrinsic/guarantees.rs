use crate::constants::{CORES_COUNT, MAX_DEPENDENCY_ITEMS};
use crate::types::{
    TimeSlot, ValidatorIndex, CoreIndex, Hash, GuaranteesExtrinsic, AvailabilityAssignments, ReportErrorCode, OutputDataReports, ProcessError
};
use crate::blockchain::state::get_recent_history;
use crate::utils::common::is_sorted_and_unique;

impl GuaranteesExtrinsic {
    /// The guarantees extrinsic is a series of guarantees, at most one for each core, each of which is 
    // a tuple of a work-report, a credential and its corresponding timeslot. The core index of each 
    // guarantee must be unique and guarantees must be in ascending order of this.
    // They are reports of newly completed workloads whose accuracy is guaranteed by specific validators. 
    // A work-package, which comprises several work items, is transformed by validators acting as guarantors 
    // into its corresponding workreport, which similarly comprises several work outputs and then presented 
    // on-chain within the guarantees extrinsic.
    pub fn process(
        &self, 
        assurances_state: &mut AvailabilityAssignments, 
        post_tau: &TimeSlot) 
    -> Result<OutputDataReports, ProcessError> {

        // At most one guarantee for each core
        if self.report_guarantee.len() > CORES_COUNT {
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyGuarantees));
        }

        // There must be no duplicate work-package hashes (i.e. two work-reports of the same package).
        let mut packages_hashes = self.report_guarantee.iter()
                                                                        .map(|i| i.report.package_spec.hash)
                                                                        .collect::<Vec<_>>();
        packages_hashes.sort(); 
        if !is_sorted_and_unique(&packages_hashes) {
            return Err(ProcessError::ReportError(ReportErrorCode::DuplicatePackage));
        }
        
        // Therefore, we require the cardinality of all work-packages to be the length of the work-report sequence
        if packages_hashes.len() != self.report_guarantee.len() {
            return Err(ProcessError::ReportError(ReportErrorCode::LengthNotEqual));
        }

        // We limit the sum of the number of items in the segment-root lookup dictionary and the number of prerequisites to MAX_DEPENDENCY_ITEMS
        for guarantee in &self.report_guarantee {
            if guarantee.report.context.prerequisites.len() + guarantee.report.segment_root_lookup.len() > MAX_DEPENDENCY_ITEMS {
                return Err(ProcessError::ReportError(ReportErrorCode::TooManyDependencies));
            }
        }

        let mut reported = Vec::new();
        let mut reporters = Vec::new();
        let mut core_index: Vec<CoreIndex> = Vec::new();
        let recent_history = get_recent_history();

        let packages_map: std::collections::HashMap<Hash, Hash> = self.report_guarantee
                                                                    .iter()
                                                                    .map(|g| 
                                                                            (g.report.package_spec.hash, g.report.package_spec.exports_root))
                                                                    .collect();
        
        let recent_history_map: std::collections::HashMap<_, _> = recent_history.blocks
            .iter()
            .flat_map(|blocks| blocks.reported.0.iter())
            .map(|report| (report.hash, report.exports_root))
            .collect();

        for guarantee in &self.report_guarantee {
       
            // The core index of each guarantee must be unique and guarantees must be in ascending order of this
            core_index.push(guarantee.report.core_index);
            if !is_sorted_and_unique(&core_index) {
                return Err(ProcessError::ReportError(ReportErrorCode::OutOfOrderGuarantee));
            }

            if guarantee.report.core_index > CORES_COUNT as CoreIndex {
                return Err(ProcessError::ReportError(ReportErrorCode::BadCoreIndex));
            }

            // The credential is a sequence of two or three tuples of a unique validator index and a signature
            if guarantee.signatures.len() < 2 || guarantee.signatures.len() > 3 {
                return Err(ProcessError::ReportError(ReportErrorCode::InsufficientGuarantees));
            }

            // Credentials must be ordered by their validator index
            let validator_indexes: Vec<ValidatorIndex> = guarantee.signatures.iter().map(|i| i.validator_index).collect();
            if !is_sorted_and_unique(&validator_indexes) {
                return Err(ProcessError::ReportError(ReportErrorCode::NotSortedOrUniqueGuarantors));
            }

            // We require that the work-package of the report not be the work-package of some other report made in the past.
            if recent_history_map.contains_key(&guarantee.report.package_spec.hash) {
                return Err(ProcessError::ReportError(ReportErrorCode::DuplicatePackage));
            }
            // We ensure that the work-package not appear anywhere within our pipeline.
            //let assignments = get_staging_reporting_assurance();
            for i in 0..CORES_COUNT {
                if let Some(assignment) = &assurances_state.0[i] {
                    if assignment.report.package_spec.hash == guarantee.report.package_spec.hash {
                        return Err(ProcessError::ReportError(ReportErrorCode::DuplicatePackage));
                    }
                }
            }
 
            // We require that the prerequisite work-packages, if present, be either in the extrinsic or in our recent history 
            for prerequisite in &guarantee.report.context.prerequisites {
                if !packages_map.contains_key(prerequisite) && !recent_history_map.contains_key(prerequisite) {
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
                    _ => return Err(ProcessError::ReportError(ReportErrorCode::SegmentRootLookupInvalid)),
                }
            }

            // Process the work report
            let OutputDataReports {
                reported: new_reported,
                reporters: new_reporters,
            } = guarantee.report.process(assurances_state, post_tau, guarantee.slot, &guarantee.signatures)?;
    
            reported.extend(new_reported);
            reporters.extend(new_reporters);
        }
    
        reported.sort_by(|a, b| a.work_package_hash.cmp(&b.work_package_hash));
        reporters.sort();
    
        Ok(OutputDataReports { reported, reporters })
    }
    
}

