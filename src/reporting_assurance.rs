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

use crate::codec::disputes_extrinsic::{AvailabilityAssignments};

// The state of the reporting and availability portion of the protocol is largely contained within ρ, which tracks the 
// work-reports which have been reported but are not yet known to be available to a super-majority of validators, together 
// with the time at which each was reported. As mentioned earlier, only one report may be assigned to a core at any given time.
static REPORT_AVAILABILITY_STATE: Lazy<Mutex<Option<AvailabilityAssignments>>> = Lazy::new(|| Mutex::new(None));

