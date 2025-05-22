use crate::types::{Gas, RamAddress, Balance};

// The size of the on-chain entropy pool
pub const ENTROPY_POOL_SIZE: usize = 4;

// Total number of validators
pub const VALIDATORS_COUNT: usize = 1023;
//pub const VALIDATORS_COUNT: usize = 6;

// The length of an epoch timeslots.
pub const EPOCH_LENGTH: usize = 600;
//pub const EPOCH_LENGTH: usize = 12;

// Total number of cores
pub const CORES_COUNT: usize = 341;
//pub const CORES_COUNT: usize = 2;

// The rotation period of validator-core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;
//pub const ROTATION_PERIOD: u32 = 10;

// The number of slots into an epoch at which ticket-submission ends
//pub const TICKET_SUBMISSION_ENDS: usize = 500;
pub const TICKET_SUBMISSION_ENDS: usize = 10;

// The number of ticket entries per validator.
//pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 3;

// The maximum number of tickets which may be submitted in a single extrinsic.
pub const MAX_TICKETS_PER_EXTRINSIC: usize = 3;
//pub const MAX_TICKETS_PER_EXTRINSIC: usize = 16;

// Validator super majority
pub const VALIDATORS_SUPER_MAJORITY: usize = (VALIDATORS_COUNT * 2) / 3 + 1;

// One third validators
pub const ONE_THIRD_VALIDATORS: usize = VALIDATORS_COUNT / 3;

pub const AVAIL_BITFIELD_BYTES: usize = (CORES_COUNT + 7) / 8;


// The additional minimum balance required per item of elective service state
pub const MIN_BALANCE_PER_ITEM: Balance = 10;

// The additional minimum balance required per octet of elective service state.
pub const MIN_BALANCE_PER_OCTET: Balance = 1;

// The basic minimum balance which all services require.
pub const MIN_BALANCE: Balance = 100;

// The size of recent history in blocks
pub const RECENT_HISTORY_SIZE: usize = 8;

// The maximum number of items in the authorizations pool.
pub const MAX_ITEMS_AUTHORIZATION_POOL: usize = 8;

// The number of items in the authorizations queue.
pub const MAX_ITEMS_AUTHORIZATION_QUEUE: usize = 80;

// The maximum sum of dependency items in a work-report.
pub const MAX_DEPENDENCY_ITEMS: usize = 8;

// The maximum amount of work items in a package.
pub const MAX_WORK_ITEMS: usize = 16;

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

// The period in timeslots after which an unreferenced preimage may be expunged.
pub const MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE: u32 = 19_200;


// The number of the registers in the PVM
pub const NUM_REG: usize = 13;

// The size of ram page (Zp)
pub const PAGE_SIZE: RamAddress = 1 << 12;

// The standard pvm program initialization input data size. (Zi)
pub const PVM_INIT_INPUT_DATA_SIZE: RamAddress = 1 << 24;
#[allow(non_upper_case_globals)]
pub const Zi: u64 = 1 << 24;

// The standard pvm program initialization zone size (Zz)
pub const PVM_INIT_ZONE_SIZE: RamAddress = 1 << 16;
#[allow(non_upper_case_globals)]
pub const Zz: u64 = 1 << 16;

// The size of ram
pub const RAM_SIZE: u64 = 1 << 32;

// The hightest address of ram
pub const RAM_HIGHEST_ADDRESS: u32 = u32::MAX;

// The number of pages in ram
pub const NUM_PAGES: RamAddress = (RAM_SIZE / PAGE_SIZE as u64) as RamAddress;

pub const LOWEST_ACCESIBLE_PAGE: RamAddress = (1 << 16) / PAGE_SIZE as RamAddress;

// Jump aligment factor
pub const JUMP_ALIGNMENT: usize = 2;

// The number of erasure-coded pieces in a segment.
pub const SEGMENT_PIECES: usize = 6;

// The basic size of erasure-coded pieces in octets.
pub const PIECE_SIZE: usize = 684;

// The size of a segment in octets.
pub const SEGMENT_SIZE: usize = PIECE_SIZE * SEGMENT_PIECES;

// The size of a transfer memo in octets
pub const TRANSFER_MEMO_SIZE: usize = 128;

// Host Call result constants
pub const NONE: u64 = u64::MAX;
pub const WHAT: u64 = u64::MAX - 1;
pub const OOB: u64 = u64::MAX - 2;
pub const WHO: u64 = u64::MAX - 3;
pub const FULL: u64 = u64::MAX - 4;
pub const CORE: u64 = u64::MAX - 5;
pub const CASH: u64 = u64::MAX - 6;
pub const LOW: u64 = u64::MAX - 7;
pub const HUH: u64 = u64::MAX - 8;
pub const OK: u64 = 0;

// Inner PVM result codes
pub const HALT: usize = 0;
pub const PANIC: usize = 1;
pub const FAULT: usize = 2;
pub const HOST: usize = 3;
pub const OOG: usize = 4;


pub const AUTH_POOLS: u8 = 1;
pub const AUTH_QUEUE: u8 = 2;
pub const RECENT_HISTORY: u8 = 3;
pub const SAFROLE: u8 = 4;
pub const DISPUTES: u8 = 5;
pub const ENTROPY: u8 = 6;
pub const NEXT_VALIDATORS: u8 = 7;
pub const CURR_VALIDATORS: u8 = 8;
pub const PREV_VALIDATORS: u8 = 9;
pub const AVAILABILITY: u8 = 10;
pub const TIME: u8 = 11;
pub const PRIVILEGES: u8 = 12;
pub const STATISTICS: u8 = 13;
pub const READY_QUEUE: u8 = 14;
pub const ACCUMULATION_HISTORY: u8 = 15;


