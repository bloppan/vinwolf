type RamAddress = u32;

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
// Lowest accessible page
pub const LOWEST_ACCESIBLE_PAGE: RamAddress = (1 << 16) / PAGE_SIZE as RamAddress;
// Jump aligment factor
pub const JUMP_ALIGNMENT: usize = 2;

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

// PVM Instructions
pub const TRAP: u8 = 0;
pub const FALLTHROUGH: u8 = 1;
pub const ECALLI: u8 = 10;
pub const LOAD_IMM_64: u8 = 20;
pub const STORE_IMM_U8: u8 = 30;
pub const STORE_IMM_U16: u8 = 31;
pub const STORE_IMM_U32: u8 = 32;
pub const STORE_IMM_U64: u8 = 33;
pub const JUMP: u8 = 40;
pub const JUMP_IND: u8 = 50;
pub const LOAD_IMM: u8 = 51;
pub const LOAD_U8: u8 = 52;
pub const LOAD_I8: u8 = 53;
pub const LOAD_U16: u8 = 54;
pub const LOAD_I16: u8 = 55;
pub const LOAD_U32: u8 = 56;
pub const LOAD_I32: u8 = 57;
pub const LOAD_U64: u8 = 58;
pub const STORE_U8: u8 = 59;
pub const STORE_U16: u8 = 60;
pub const STORE_U32: u8 = 61;
pub const STORE_U64: u8 = 62;
pub const STORE_IMM_IND_U8: u8 = 70;
pub const STORE_IMM_IND_U16: u8 = 71;
pub const STORE_IMM_IND_U32: u8 = 72;
pub const STORE_IMM_IND_U64: u8 = 73;
pub const LOAD_IMM_JUMP: u8 = 80;
pub const BRANCH_EQ_IMM: u8 = 81;
pub const BRANCH_NE_IMM: u8 = 82;
pub const BRANCH_LT_U_IMM: u8 = 83;
pub const BRANCH_LE_U_IMM: u8 = 84;
pub const BRANCH_GE_U_IMM: u8 = 85;
pub const BRANCH_GT_U_IMM: u8 = 86;
pub const BRANCH_LT_S_IMM: u8 = 87;
pub const BRANCH_LE_S_IMM: u8 = 88;
pub const BRANCH_GE_S_IMM: u8 = 89;
pub const BRANCH_GT_S_IMM: u8 = 90;
pub const MOVE_REG: u8 = 100;
pub const SBRK: u8 = 101;
pub const COUNT_SET_BITS_64: u8 = 102;
pub const COUNT_SET_BITS_32: u8 = 103;
pub const LEADING_ZERO_BITS_64: u8 = 104;
pub const LEADING_ZERO_BITS_32: u8 = 105;
pub const TRAILING_ZERO_BITS_64: u8 = 106;
pub const TRAILING_ZERO_BITS_32: u8 = 107;
pub const SIGN_EXTEND_8: u8 = 108;
pub const SIGN_EXTEND_16: u8 = 109;
pub const ZERO_EXTEND_16: u8 = 110;
pub const REVERSE_BYTES: u8 = 111;
pub const STORE_IND_U8: u8 = 120;
pub const STORE_IND_U16: u8 = 121;
pub const STORE_IND_U32: u8 = 122;
pub const STORE_IND_U64: u8 = 123;
pub const LOAD_IND_U8: u8 = 124;
pub const LOAD_IND_I8: u8 = 125;
pub const LOAD_IND_U16: u8 = 126;
pub const LOAD_IND_I16: u8 = 127;
pub const LOAD_IND_U32: u8 = 128;
pub const LOAD_IND_I32: u8 = 129;
pub const LOAD_IND_U64: u8 = 130;
pub const ADD_IMM_32: u8 = 131;
pub const AND_IMM: u8 = 132;
pub const XOR_IMM: u8 = 133;
pub const OR_IMM: u8 = 134;
pub const MUL_IMM_32: u8 = 135;
pub const SET_LT_U_IMM: u8 = 136;
pub const SET_LT_S_IMM: u8 = 137;
pub const SHLO_L_IMM_32: u8 = 138;
pub const SHLO_R_IMM_32: u8 = 139;
pub const SHAR_R_IMM_32: u8 = 140;
pub const NEG_ADD_IMM_32: u8 = 141;
pub const SET_GT_U_IMM: u8 = 142;
pub const SET_GT_S_IMM: u8 = 143;
pub const SHLO_L_IMM_ALT_32: u8 = 144;
pub const SHLO_R_IMM_ALT_32: u8 = 145;
pub const SHAR_R_IMM_ALT_32: u8 = 146;
pub const CMOV_IZ_IMM: u8 = 147;
pub const CMOV_NZ_IMM: u8 = 148;
pub const ADD_IMM_64: u8 = 149;
pub const MUL_IMM_64: u8 = 150;
pub const SHLO_L_IMM_64: u8 = 151;
pub const SHLO_R_IMM_64: u8 = 152;
pub const SHAR_R_IMM_64: u8 = 153;
pub const NEG_ADD_IMM_64: u8 = 154;
pub const SHLO_L_IMM_ALT_64: u8 = 155;
pub const SHLO_R_IMM_ALT_64: u8 = 156;
pub const SHAR_R_IMM_ALT_64: u8 = 157;
pub const ROT_R_64_IMM: u8 = 158;
pub const ROT_R_64_IMM_ALT: u8 = 159;
pub const ROT_R_32_IMM: u8 = 160;
pub const ROT_R_32_IMM_ALT: u8 = 161;
pub const BRANCH_EQ: u8 = 170;
pub const BRANCH_NE: u8 = 171;
pub const BRANCH_LT_U: u8 = 172;
pub const BRANCH_LT_S: u8 = 173;
pub const BRANCH_GE_U: u8 = 174;
pub const BRANCH_GE_S: u8 = 175;
pub const LOAD_IMM_JUMP_IND: u8 = 180;
pub const ADD_32: u8 = 190;
pub const SUB_32: u8 = 191;
pub const MUL_32: u8 = 192;
pub const DIV_U_32: u8 = 193;
pub const DIV_S_32: u8 = 194;
pub const REM_U_32: u8 = 195;
pub const REM_S_32: u8 = 196;
pub const SHLO_L_32: u8 = 197;
pub const SHLO_R_32: u8 = 198;
pub const SHAR_R_32: u8 = 199;
pub const ADD_64: u8 = 200;
pub const SUB_64: u8 = 201;
pub const MUL_64: u8 = 202;
pub const DIV_U_64: u8 = 203;
pub const DIV_S_64: u8 = 204;
pub const REM_U_64: u8 = 205;
pub const REM_S_64: u8 = 206;
pub const SHLO_L_64: u8 = 207;
pub const SHLO_R_64: u8 = 208;
pub const SHAR_R_64: u8 = 209;
pub const AND: u8 = 210;
pub const XOR: u8 = 211;
pub const OR: u8 = 212;
pub const MUL_UPPER_S_S: u8 = 213;
pub const MUL_UPPER_U_U: u8 = 214;
pub const MUL_UPPER_S_U: u8 = 215;
pub const SET_LT_U: u8 = 216;
pub const SET_LT_S: u8 = 217;
pub const CMOV_IZ: u8 = 218;
pub const CMOV_NZ: u8 = 219;
pub const ROT_L_64: u8 = 220;
pub const ROT_L_32: u8 = 221;
pub const ROT_R_64: u8 = 222;
pub const ROT_R_32: u8 = 223;
pub const AND_INV: u8 = 224;
pub const OR_INV: u8 = 225;
pub const XNOR: u8 = 226;
pub const MAX: u8 = 227;
pub const MAX_U: u8 = 228;
pub const MIN: u8 = 229;
pub const MIN_U: u8 = 230;

