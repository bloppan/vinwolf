pub const NUM_VALIDATORS: usize = 6;
pub const EPOCH_LENGTH: usize = 12;
pub const NUM_CORES: usize = 2;


pub const VALIDATORS_SUPER_MAJORITY: usize = NUM_VALIDATORS * 2/3 + 1;
pub const AVAIL_BITFIELD_BYTES: usize = (NUM_CORES + 7) / 8;

