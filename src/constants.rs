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

pub const AVAIL_BITFIELD_BYTES: usize = (CORES_COUNT + 7) / 8;

// The size of recent history in blocks
pub const RECENT_HISTORY_SIZE: usize = 8;