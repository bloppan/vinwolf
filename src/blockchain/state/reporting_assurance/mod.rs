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

use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::{
    Hash, AssurancesExtrinsic, AvailabilityAssignment, AvailabilityAssignments, GuaranteesExtrinsic, TimeSlot, CoreIndex
};
use crate::blockchain::block::extrinsic::assurances::{OutputDataAssurances, ErrorCode as AssurancesErrorCode};
use crate::utils::codec::work_report::{OutputData, ReportErrorCode};

use super::ProcessError;

pub mod refine_context;
pub mod work_report;
pub mod work_item;
pub mod work_package;
pub mod work_result;

// The state of the reporting and availability portion of the protocol is largely contained within ρ, which tracks the 
// work-reports which have been reported but are not yet known to be available to a super-majority of validators, together 
// with the time at which each was reported. As mentioned earlier, only one report may be assigned to a core at any given time.
static REPORT_AVAILABILITY_STATE: Lazy<Mutex<AvailabilityAssignments>> = Lazy::new(|| Mutex::new(AvailabilityAssignments{assignments: Box::new(std::array::from_fn(|_| None))}));

pub fn set_reporting_assurance_staging_state(post_state: &AvailabilityAssignments) {
    let mut state = REPORT_AVAILABILITY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_reporting_assurance_staging_state() -> AvailabilityAssignments {
    let state = REPORT_AVAILABILITY_STATE.lock().unwrap(); 
    return state.clone();
}

pub fn add_assignment(assignment: &AvailabilityAssignment) {
    let mut state = REPORT_AVAILABILITY_STATE.lock().unwrap();
    state.assignments[assignment.report.core_index as usize] = Some(assignment.clone());
}

pub fn remove_assignment(core_index: &CoreIndex) {
    let mut state = REPORT_AVAILABILITY_STATE.lock().unwrap();
    state.assignments[*core_index as usize] = None;
}

pub fn process_guarantees(
    assurances_state: &mut AvailabilityAssignments, 
    guarantees: &GuaranteesExtrinsic, 
    post_tau: &TimeSlot) 
-> Result<OutputData, ProcessError> {

    //let stg_assurances_state = assurances_state.clone();
    set_reporting_assurance_staging_state(&assurances_state.clone());
    //println!("assurances pre: {:0x?}", assurances_state);
    let output_data = guarantees.process(post_tau)?;
    //println!("output_data = {:0x?}", output_data);
    *assurances_state = get_reporting_assurance_staging_state();
    //println!("asssurances post: {:0x?}", assurances_state);
    Ok(OutputData {
        reported: output_data.reported,
        reporters: output_data.reporters,
    })
}

pub fn process_assurances(
    assurances_state: &mut AvailabilityAssignments, 
    assurances: &AssurancesExtrinsic, 
    post_tau: &TimeSlot,
    parent: &Hash) 
-> Result<OutputDataAssurances, AssurancesErrorCode> {

    //let stg_assurances_state = assurances_state.clone();
    set_reporting_assurance_staging_state(&assurances_state.clone());
    //println!("assurances pre: {:0x?}", assurances_state);
    let output_data = assurances.process(post_tau, parent)?;
    //println!("output_data = {:0x?}", output_data);
    *assurances_state = get_reporting_assurance_staging_state();
    //println!("asssurances post: {:0x?}", assurances_state);
    Ok(OutputDataAssurances {
        reported: output_data.reported,
    })
}
