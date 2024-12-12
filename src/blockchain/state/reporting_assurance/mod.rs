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

use frame_support::sp_runtime::offchain::storage_lock::Time;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use sp_core::keccak_256;

use crate::codec::{Encode};
use crate::codec::block;
use crate::codec::refine_context::RefineContext;
use crate::types::{Entropy, OpaqueHash, TimeSlot, WorkPackageHash, Ed25519Public};
use crate::constants::{CORES_COUNT, EPOCH_LENGTH, ROTATION_PERIOD, VALIDATORS_COUNT, WORK_REPORT_TIMEOUT};
use crate::codec::disputes_extrinsic::{AvailabilityAssignments, AvailabilityAssignment};
use crate::blockchain::block::extrinsic::guarantees::GuaranteesExtrinsic;
use crate::codec::work_report::{ReportedPackage, OutputData, OutputWorkReport, AuthPool, AuthPools, ErrorCode};
use crate::codec::history::{Mmr, BlockInfo};
use crate::shuffle::shuffle;
use crate::trie::mmr_super_peak;
use crate::blockchain::state::validators::{get_validators_state, ValidatorSet};
use crate::blockchain::state::authorization::{get_authpool_state, set_authpool_state};
use crate::blockchain::state::recent_history::get_history_state;
use crate::blockchain::state::time::get_time_state;


mod work_report;

// The state of the reporting and availability portion of the protocol is largely contained within ρ, which tracks the 
// work-reports which have been reported but are not yet known to be available to a super-majority of validators, together 
// with the time at which each was reported. As mentioned earlier, only one report may be assigned to a core at any given time.
static REPORT_AVAILABILITY_STATE: Lazy<Mutex<AvailabilityAssignments>> = Lazy::new(|| Mutex::new(AvailabilityAssignments{assignments: Box::new(std::array::from_fn(|_| None))}));

pub fn set_reporting_assurance_state(post_state: &AvailabilityAssignments) {
    let mut state = REPORT_AVAILABILITY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_reporting_assurance_state() -> AvailabilityAssignments {
    let state = REPORT_AVAILABILITY_STATE.lock().unwrap(); 
    return state.clone();
}

pub fn add_assignment(assignment: &AvailabilityAssignment) {
    let mut state = REPORT_AVAILABILITY_STATE.lock().unwrap();
    state.assignments[assignment.report.core_index as usize] = Some(assignment.clone());
}

fn place_reports(guarantees: &GuaranteesExtrinsic, slot: TimeSlot) -> Result<OutputData, ErrorCode> {

    let mut reported: Vec<ReportedPackage> = Vec::new();
    let mut reporters: Vec<Ed25519Public> = Vec::new();

    for guarantee in &guarantees.report_guarantee {
        
        guarantee.report.validate_authorization()?;
        guarantee.report.is_recent()?;
        
        let OutputData {
            reported: new_reported,
            reporters: new_reporters,
        } = guarantee.report.try_place(guarantee.slot, &guarantee.signatures)?;

        reported.extend_from_slice(&new_reported);
        reporters.extend_from_slice(&new_reporters);
       
    }

    reporters.sort();

    return Ok(OutputData{reported: reported, reporters: reporters});
}

pub fn process_report_assurance(guarantees: &GuaranteesExtrinsic, slot: TimeSlot) -> Result<OutputData, ErrorCode> {

    // Work report - is valid?
    // Work report - is recent?
    if guarantees.report_guarantee.len() > CORES_COUNT {
        return Err(ErrorCode::TooManyGuarantees);
    }

    let mut reported: Vec<ReportedPackage> = Vec::new();
    let mut reporters: Vec<Ed25519Public> = Vec::new();

    let OutputData {
        reported: new_reported,
        reporters: new_reporters,
    } = place_reports(guarantees, slot)?;

    reported.extend_from_slice(&new_reported);
    reporters.extend_from_slice(&new_reporters);

    Ok(OutputData {
        reported,
        reporters,
    })
}


fn rotation(c: &[u16], n: u16) -> Vec<u16> {

    let mut result: Vec<u16> = Vec::with_capacity(c.len());

    for x in c {
        result.push((x + n) % CORES_COUNT as u16);
    }

    return result;
}

fn permute(entropy: &Entropy, t: u16) -> Vec<u16> {

    let mut items: Vec<u16> = Vec::with_capacity(VALIDATORS_COUNT);

    for i in 0..VALIDATORS_COUNT {
        items.push(((CORES_COUNT * i) / VALIDATORS_COUNT) as u16);
    }

    let res_shuffle = shuffle(&items, entropy);
    let n = t % EPOCH_LENGTH as u16 / ROTATION_PERIOD as u16;
    rotation(&res_shuffle, n)
}

//fn guarantor_assignments(assignmets: &[u16], )

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