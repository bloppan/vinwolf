/*
    TINY CONFIG
*/

// Total number of validators
pub const VALIDATORS_COUNT: usize = 6;
// The length of an epoch timeslots.
pub const EPOCH_LENGTH: usize = 12;
// Total number of cores
pub const CORES_COUNT: usize = 2;
// The rotation period of validator-core assignments, in timeslots.
pub const ROTATION_PERIOD: u32 = 4;
// The number of slots into an epoch at which ticket-submission ends
pub const TICKET_SUBMISSION_ENDS: usize = 10;
// The number of ticket entries per validator.
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 3;
// The maximum number of tickets which may be submitted in a single extrinsic.
pub const MAX_TICKETS_PER_EXTRINSIC: usize = 3;
// The period in timeslots after which an unreferenced preimage may be expunged.
pub const MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE: u32 = 32;


/*
    FULL CONFIG
*/
/*pub const VALIDATORS_COUNT: usize = 1023;
pub const EPOCH_LENGTH: usize = 600;
pub const CORES_COUNT: usize = 341;
pub const ROTATION_PERIOD: u32 = 10;
pub const TICKET_SUBMISSION_ENDS: usize = 500;
pub const TICKET_ENTRIES_PER_VALIDATOR: u8 = 2;
pub const MAX_TICKETS_PER_EXTRINSIC: usize = 16;
pub const MAX_TIMESLOTS_AFTER_UNREFEREND_PREIMAGE: u32 = 19_200;*/


// The size of the on-chain entropy pool
pub const ENTROPY_POOL_SIZE: usize = 4;
// Validator super majority
pub const VALIDATORS_SUPER_MAJORITY: usize = (VALIDATORS_COUNT * 2) / 3 + 1;
// One third validators
pub const ONE_THIRD_VALIDATORS: usize = VALIDATORS_COUNT / 3;
// Available bitfield bytes
pub const AVAIL_BITFIELD_BYTES: usize = (CORES_COUNT + 7) / 8;
// The additional minimum balance required per item of elective service state
pub const MIN_BALANCE_PER_ITEM: u64 = 10;
// The additional minimum balance required per octet of elective service state.
pub const MIN_BALANCE_PER_OCTET: u64 = 1;
// The basic minimum balance which all services require.
pub const MIN_BALANCE: u64 = 100;
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
// The gas allocated to invoke a work-report's Accumulation logic
pub const WORK_REPORT_GAS_LIMIT: i64 = 10_000_000;
// The gas allocated to invoke a work-package's Is-Authorized logic.
pub const WORK_PACKAGE_GAS_LIMIT: i64 = 50_000_000;
// The gas allocated to invoke a work-package's Refine logic.
pub const WORK_PACKAGE_REFINE_GAS: i64 = 5_000_000_000;
// The total gas allocated across for all Accumulation.
pub const TOTAL_GAS_ALLOCATED: i64 = 3_500_000_000;
// The maximum age in timeslots of the lookup anchor.
pub const MAX_AGE_LOOKUP_ANCHOR: u32 = 14_400;
// The maximum size of service code in octets
pub const MAX_SERVICE_CODE_SIZE: usize = 4_000_000;
// The slot period, in seconds.
pub const SLOT_PERIOD: usize = 6;
// The maximum number of entries in the accumulation queue.
pub const MAX_ENTRIES_IN_ACC_QUEUE: usize = 1024;
// The maximum number of extrinsics in a work-package.
pub const MAX_EXTRINSICS_IN_WP: usize = 128;
// The period in timeslots after which reported but unavailable work may be replaced.
pub const REPORTED_WORK_REPLACEMENT_PERIOD: usize = 5;
// The maximum size of is-authorized code in octets.
pub const MAX_IS_AUTHORIZED_SIZE: usize = 64_000;
// The maximum size of an encoded work-package together with its extrinsic data and import implications, in octets.
pub const MAX_ENCODED_WORK_PACKAGE_SIZE: u64 = 12 * (1 << 20);
// The number of erasure-coded pieces in a segment.
pub const SEGMENT_PIECES: usize = 6;
// The basic size of erasure-coded pieces in octets.
pub const PIECE_SIZE: usize = 684;
// The size of a segment in octets.
pub const SEGMENT_SIZE: usize = PIECE_SIZE * SEGMENT_PIECES;
// The maximum number of imports in a work-package.
pub const MAX_WORK_PACKAGE_IMPORTS: usize = 3_072;
// The maximum total size of all unbounded blobs in a work-report, in octets.
pub const MAX_WORK_REPORT_TOTAL_SIZE: u64 = 48 * (1 << 10);
// The maximum number of exports in a work-package.
pub const MAX_WORK_PACKAGE_EXPORTS: usize = 3_072;
// The size of a transfer memo in octets
pub const TRANSFER_MEMO_SIZE: usize = 128;
// JAM global state constants
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
pub const RECENT_ACC_OUTPUTS: u8 = 16;

