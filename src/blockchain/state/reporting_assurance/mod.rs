// Reporting and assurance are the two on-chain processes we do to allow the results of in-core computation to make
// its way into the service state singleton. A work-package, which comprises several work items, is transformed 
// by validators acting as guarantors into its corresponding work-report, which similarly comprises several work outputs 
// and then presented on-chain within the guarantees extrinsic. At this point, the work-package is erasure coded into a
// multitude of segments and each segment distributed to the associated validator who then attests to its availability 
// through an assurance placed on-chain. After enough assurances the work-report is considered available, and the work 
// outputs transform the state of their associated service by virtue of accumulation. The report may also be timed-out, 
// implying it may be replaced by another report without accumulation.

// From the perspective of the work-report, therefore, the guarantee happens first and the assurance afterwards. However, 
// from the perspective of a block's statetransition, the assurances are best processed first since each core may only 
// have a single work-report pending its package becoming available at a time. Thus, we will first cover the transition 
// arising from processing the availability assurances followed by the work-report guarantees. This synchroneity can be 
// seen formally through the requirement of an intermediate state ρ‡.

use crate::types::{
    Hash, AssurancesExtrinsic, AvailabilityAssignment, AvailabilityAssignments, GuaranteesExtrinsic, TimeSlot, CoreIndex,
    OutputDataAssurances, OutputDataReports
};
use super::ProcessError;

pub mod work_report;
pub mod work_result;

// The state of the reporting and availability portion of the protocol is largely contained within ρ, which tracks the 
// work-reports which have been reported but are not yet known to be available to a super-majority of validators, together 
// with the time at which each was reported. As mentioned earlier, only one report may be assigned to a core at any given time.

pub fn add_assignment(assignment: &AvailabilityAssignment, state: &mut AvailabilityAssignments) {
    state.0[assignment.report.core_index as usize] = Some(assignment.clone());
}

pub fn remove_assignment(core_index: &CoreIndex, state: &mut AvailabilityAssignments) {
    state.0[*core_index as usize] = None;
}

pub fn process_assurances(
    assurances_state: &mut AvailabilityAssignments, 
    assurances: &AssurancesExtrinsic, 
    post_tau: &TimeSlot,
    parent: &Hash) 
-> Result<OutputDataAssurances, ProcessError> {

    let output_data = assurances.process(assurances_state, post_tau, parent)?;

    Ok(OutputDataAssurances {
        reported: output_data.reported,
    })
}

pub fn process_guarantees(
    assurances_state: &mut AvailabilityAssignments, 
    guarantees: &GuaranteesExtrinsic, 
    post_tau: &TimeSlot) 
-> Result<OutputDataReports, ProcessError> {

    let output_data = guarantees.process(assurances_state, post_tau)?;

    Ok(OutputDataReports {
        reported: output_data.reported,
        reporters: output_data.reporters,
    })
}

