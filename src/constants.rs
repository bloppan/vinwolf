use crate::types::Gas;

// Total number of validators
pub const VALIDATORS_COUNT: usize = 6;

// The length of an epoch timeslots.
pub const EPOCH_LENGTH: usize = 12;

// The number of slots into an epoch at which ticket-submission ends
pub const TICKET_SUBMISSION_ENDS: usize = 10;

// Total number of cores
pub const CORES_COUNT: usize = 2;

// Validator super majority
pub const VALIDATORS_SUPER_MAJORITY: usize = (VALIDATORS_COUNT * 2) / 3 + 1;

// One third validators
pub const ONE_THIRD_VALIDATORS: usize = VALIDATORS_COUNT / 3;

pub const AVAIL_BITFIELD_BYTES: usize = (CORES_COUNT + 7) / 8;

// The size of recent history in blocks
pub const RECENT_HISTORY_SIZE: usize = 8;

// The maximum number of items in the authorizations pool.
pub const MAX_ITEMS_AUTHORIZATION_POOL: usize = 8;

// The number of items in the authorizations queue.
pub const MAX_ITEMS_AUTHORIZATION_QUEUE: usize = 80;

// The rotation period of validator-core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;

// The period in timeslots after which reported but unavailable work may be replaced.
pub const WORK_REPORT_TIMEOUT: u32 = 5;

// The gas allocated to invoke a work-report's Accumulation logic
pub const WORK_REPORT_GAS_LIMIT: Gas = 10_000_000;

// The gas allocated to invoke a work-package's Is-Authorized logic.
pub const WORK_PACKAGE_GAS_LIMIT: Gas = 50_000_000;

// The gas allocated to invoke a work-package's Refine logic.
pub const WORK_PACKAGE_REFINE_GAS: Gas = 5_000_000_000;

// The total gas allocated across for all Accumulation.
pub const TOTAL_GAS_ALLOCATED: Gas = 3_500_000_000;

