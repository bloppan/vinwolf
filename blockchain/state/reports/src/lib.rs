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

use jam_types::{
    AvailabilityAssignments, EntropyPool, Hash, OutputDataAssurances, OutputDataReports, TimeSlot, ValidatorsData, ProcessError, Guarantee
};
use block::extrinsic;

pub mod assurances {

    use jam_types::Assurance;

    use super::*;

    pub fn process(
        availability_state: &mut AvailabilityAssignments, 
        assurances: &[Assurance], 
        post_tau: &TimeSlot,
        parent: &Hash) 
    -> Result<OutputDataAssurances, ProcessError> {
        
        let output_data = extrinsic::assurances::process(assurances, availability_state, post_tau, parent)?;

        Ok(OutputDataAssurances {
            reported: output_data.reported,
        })
    }
}

pub mod guarantees {

    use super::*;

    pub fn process(
        availability_state: &mut AvailabilityAssignments, 
        guarantees_extrinsic: &[Guarantee], 
        post_tau: &TimeSlot,
        entropy_pool: &EntropyPool,
        prev_validators: &ValidatorsData,
        curr_validators: &ValidatorsData) 
    -> Result<OutputDataReports, ProcessError> {

        let output_data = extrinsic::guarantees::process(guarantees_extrinsic, availability_state, post_tau, entropy_pool, prev_validators, curr_validators)?;

        Ok(OutputDataReports {
            reported: output_data.reported,
            reporters: output_data.reporters,
        })
    }
}
