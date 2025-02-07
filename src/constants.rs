use crate::types::{Gas, RamAddress};

// The size of the on-chain entropy pool
pub const ENTROPY_POOL_SIZE: usize = 4;

// Total number of validators
//pub const VALIDATORS_COUNT: usize = 1023;
pub const VALIDATORS_COUNT: usize = 6;

// The length of an epoch timeslots.
//pub const EPOCH_LENGTH: usize = 600;
pub const EPOCH_LENGTH: usize = 12;

// Total number of cores
//pub const CORES_COUNT: usize = 341;
pub const CORES_COUNT: usize = 2;

// The rotation period of validator-core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;
//pub const ROTATION_PERIOD: u32 = 10;

// The number of slots into an epoch at which ticket-submission ends
//pub const TICKET_SUBMISSION_ENDS: usize = 500;
pub const TICKET_SUBMISSION_ENDS: usize = 10;

// The number of ticket entries per validator.
//pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 3;


// Validator super majority
pub const VALIDATORS_SUPER_MAJORITY: usize = (VALIDATORS_COUNT * 2) / 3 + 1;

// One third validators
pub const ONE_THIRD_VALIDATORS: usize = VALIDATORS_COUNT / 3;

pub const AVAIL_BITFIELD_BYTES: usize = (CORES_COUNT + 7) / 8;

// The size of recent history in blocks
pub const RECENT_HISTORY_SIZE: usize = 8;

// The maximum number of items in the authorizations pool.
pub const MAX_ITEMS_AUTHORIZATION_POOL: usize = 8;

// The maximum number of tickets which may be submitted in a single extrinsic.
pub const MAX_TICKETS_PER_EXTRINSIC: usize = 16;

// The number of items in the authorizations queue.
pub const MAX_ITEMS_AUTHORIZATION_QUEUE: usize = 80;

// The maximum sum of dependency items in a work-report.
pub const MAX_DEPENDENCY_ITEMS: usize = 8;

// The maximum total size of all output blobs in a work-report, in octets
pub const MAX_OUTPUT_BLOB_SIZE: usize = 48 << 10;

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

// The maximum age of a lookup anchor in timeslots.
pub const MAX_AGE_LOOKUP_ANCHOR: u32 = 14_400;

// The number of the registers in the PVM
pub const NUM_REG: u8 = 13;

// The size of ram page
pub const PAGE_SIZE: RamAddress = 1 << 12;

// The size of ram
pub const RAM_SIZE: u64 = 1 << 32;

// The number of pages in ram
pub const NUM_PAGES: RamAddress = (RAM_SIZE / PAGE_SIZE as u64) as RamAddress;

// Jump aligment factor
pub const JUMP_ALIGNMENT: usize = 2;
