use crate::types::{Context, ExitReason, Program};
use crate::utils::codec::{BytesReader, Decode};
use crate::pvm::isa::skip;

use isa::one_offset::*;
use isa::no_arg::*;
use isa::two_reg::*;
use isa::two_reg_one_imm::*;
use isa::two_reg_two_imm::*;
use isa::two_reg_one_offset::*;
use isa::three_reg::*;
use isa::one_reg_one_ext_imm::*;
use isa::two_imm::*;
use isa::one_reg_one_imm::*;
use isa::one_reg_two_imm::*;
use isa::one_reg_one_imm_one_offset::*;

pub mod isa;
pub mod mm;
pub mod hostcall;

pub fn invoke_pvm(pvm_ctx: &mut Context, program_blob: &[u8]) -> ExitReason {
    
    let program = Program::decode(&mut BytesReader::new(program_blob)).unwrap();

    while pvm_ctx.gas > 0 {

        let exit_reason = single_step_pvm(pvm_ctx, &program);

        match exit_reason {
            ExitReason::Continue => {
                pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
            },
            ExitReason::Branch => {
              
            },
            ExitReason::PageFault(_) => {
                pvm_ctx.gas -= 1; 
                return ExitReason::page_fault;
            },
            _ => { return exit_reason; },
        }
    }

    return ExitReason::OutOfGas;
}



fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    pvm_ctx.gas -= 1;

    let next_instruction = program.code[pvm_ctx.pc as usize];
    /*println!("\nnext instruction = {next_instruction}");
    println!("pc = {:x?}", pvm_ctx.pc);
    println!("gas = {}", pvm_ctx.gas);
    println!("reg_0 = 0x{:x?}", pvm_ctx.reg[0]);
    println!("reg_1 = 0x{:x?}", pvm_ctx.reg[1]);
    println!("reg_2 = 0x{:x?}", pvm_ctx.reg[2]);
    println!("reg_3 = 0x{:x?}", pvm_ctx.reg[3]);
    println!("reg_4 = 0x{:x?}", pvm_ctx.reg[4]);
    println!("reg_5 = 0x{:x?}", pvm_ctx.reg[5]);
    println!("reg_6 = 0x{:x?}", pvm_ctx.reg[6]);
    println!("reg_7 = 0x{:x?}", pvm_ctx.reg[7]);
    println!("reg_8 = 0x{:x?}", pvm_ctx.reg[8]);
    println!("reg_9 = 0x{:x?}", pvm_ctx.reg[9]);
    println!("reg_10 = 0x{:x?}", pvm_ctx.reg[10]);
    println!("reg_11 = 0x{:x?}", pvm_ctx.reg[11]);
    println!("reg_12 = 0x{:x?}", pvm_ctx.reg[12]);*/

    let exit_reason = match next_instruction { 

        TRAP    => { trap() },
        1_u8    => { fallthrough() },
        //10_u8   => { ecalli(pvm_ctx, program) },
        20_u8   => { load_imm_64(pvm_ctx, program) },
        30_u8   => { store_imm_u8(pvm_ctx, program) },
        31_u8   => { store_imm_u16(pvm_ctx, program) },
        32_u8   => { store_imm_u32(pvm_ctx, program) },
        33_u8   => { store_imm_u64(pvm_ctx, program) },
        40_u8   => { jump(pvm_ctx, program) },
        50_u8   => { jump_ind(pvm_ctx, program) },
        51_u8   => { load_imm(pvm_ctx, program) },
        52_u8   => { load_u8(pvm_ctx, program) },
        53_u8   => { load_i8(pvm_ctx, program) },
        54_u8   => { load_u16(pvm_ctx, program) },
        55_u8   => { load_i16(pvm_ctx, program) },
        56_u8   => { load_u32(pvm_ctx, program) },
        57_u8   => { load_i32(pvm_ctx, program) },
        58_u8   => { load_u64(pvm_ctx, program) },
        59_u8   => { store_u8(pvm_ctx, program) },
        60_u8   => { store_u16(pvm_ctx, program) },
        61_u8   => { store_u32(pvm_ctx, program) },
        62_u8   => { store_u64(pvm_ctx, program) },
        70_u8   => { store_imm_ind_u8(pvm_ctx, program) },
        71_u8   => { store_imm_ind_u16(pvm_ctx, program) },
        72_u8   => { store_imm_ind_u32(pvm_ctx, program) },
        73_u8   => { store_imm_ind_u64(pvm_ctx, program) },
        80_u8   => { load_imm_jump(pvm_ctx, program) },
        81_u8   => { branch_eq_imm(pvm_ctx, program) },
        82_u8   => { branch_ne_imm(pvm_ctx, program) },
        83_u8   => { branch_lt_u_imm(pvm_ctx, program) },
        84_u8   => { branch_le_u_imm(pvm_ctx, program) },
        85_u8   => { branch_ge_u_imm(pvm_ctx, program) },
        86_u8   => { branch_gt_u_imm(pvm_ctx, program) },
        87_u8   => { branch_lt_s_imm(pvm_ctx, program) },
        88_u8   => { branch_le_s_imm(pvm_ctx, program) },
        89_u8   => { branch_ge_s_imm(pvm_ctx, program) },
        90_u8   => { branch_gt_s_imm(pvm_ctx, program) },
        100_u8  => { move_reg(pvm_ctx, program) }, 
        102_u8  => { count_set_bits_64(pvm_ctx, program) },
        103_u8  => { count_set_bits_32(pvm_ctx, program) },
        104_u8  => { leading_zero_bits_64(pvm_ctx, program) },
        105_u8  => { leading_zero_bits_32(pvm_ctx, program) },
        106_u8  => { trailing_zero_bits_64(pvm_ctx, program) },
        107_u8  => { trailing_zero_bits_32(pvm_ctx, program) },
        108_u8  => { sign_extend_8(pvm_ctx, program) },
        109_u8  => { sign_extend_16(pvm_ctx, program) },
        110_u8  => { zero_extend_16(pvm_ctx, program) },
        111_u8  => { reverse_bytes(pvm_ctx, program) },
        120_u8  => { store_ind_u8(pvm_ctx, program) },
        121_u8  => { store_ind_u16(pvm_ctx, program) },
        122_u8  => { store_ind_u32(pvm_ctx, program) },
        123_u8  => { store_ind_u64(pvm_ctx, program) },
        124_u8  => { load_ind_u8(pvm_ctx, program) },
        125_u8  => { load_ind_i8(pvm_ctx, program) },
        126_u8  => { load_ind_u16(pvm_ctx, program) },
        127_u8  => { load_ind_i16(pvm_ctx, program) },
        128_u8  => { load_ind_u32(pvm_ctx, program) },
        129_u8  => { load_ind_i32(pvm_ctx, program) },
        130_u8  => { load_ind_u64(pvm_ctx, program) },
        131_u8  => { add_imm_32(pvm_ctx, program) }, 
        132_u8  => { and_imm(pvm_ctx, program) },
        133_u8  => { xor_imm(pvm_ctx, program) },
        134_u8  => { or_imm(pvm_ctx, program) },
        135_u8  => { mul_imm_32(pvm_ctx, program) },
        136_u8  => { set_lt_u_imm(pvm_ctx, program) },
        137_u8  => { set_lt_s_imm(pvm_ctx, program) },
        138_u8  => { shlo_l_imm_32(pvm_ctx, program) },
        139_u8  => { shlo_r_imm_32(pvm_ctx, program) },
        140_u8  => { shar_r_imm_32(pvm_ctx, program) },
        141_u8  => { neg_add_imm_32(pvm_ctx, program) },
        142_u8  => { set_gt_u_imm(pvm_ctx, program) },
        143_u8  => { set_gt_s_imm(pvm_ctx, program) },
        144_u8  => { shlo_l_imm_alt_32(pvm_ctx, program) },
        145_u8  => { shlo_r_imm_alt_32(pvm_ctx, program) },
        146_u8  => { shar_r_imm_alt_32(pvm_ctx, program) },
        147_u8  => { cmov_iz_imm(pvm_ctx, program) },
        149_u8  => { add_imm_64(pvm_ctx, program) },
        150_u8  => { mul_imm_64(pvm_ctx, program) },
        151_u8  => { shlo_l_imm_64(pvm_ctx, program) },
        152_u8  => { shlo_r_imm_64(pvm_ctx, program) },
        153_u8  => { shar_r_imm_64(pvm_ctx, program) },    
        154_u8  => { neg_add_imm_64(pvm_ctx, program) },
        155_u8  => { shlo_l_imm_alt_64(pvm_ctx, program) },
        156_u8  => { shlo_r_imm_alt_64(pvm_ctx, program) },
        157_u8  => { shar_r_imm_alt_64(pvm_ctx, program) },
        158_u8  => { rot_r_64_imm(pvm_ctx, program) },
        159_u8  => { rot_r_64_imm_alt(pvm_ctx, program) },
        160_u8  => { rot_r_32_imm(pvm_ctx, program) },
        161_u8  => { rot_r_32_imm_alt(pvm_ctx, program) },
        170_u8  => { branch_eq(pvm_ctx, program) },
        171_u8  => { branch_ne(pvm_ctx, program) },
        172_u8  => { branch_lt_u(pvm_ctx, program) },
        173_u8  => { branch_lt_s(pvm_ctx, program) },
        174_u8  => { branch_ge_u(pvm_ctx, program) },
        175_u8  => { branch_ge_s(pvm_ctx, program) },
        180_u8  => { load_imm_jump_ind(pvm_ctx, program) },
        190_u8  => { add_32(pvm_ctx, program) },
        191_u8  => { sub_32(pvm_ctx, program) },
        192_u8  => { mul_32(pvm_ctx, program) },
        193_u8  => { div_u_32(pvm_ctx, program) },
        194_u8  => { div_s_32(pvm_ctx, program) },
        195_u8  => { rem_u_32(pvm_ctx, program) },
        196_u8  => { rem_s_32(pvm_ctx, program) },
        197_u8  => { shlo_l_32(pvm_ctx, program) },
        198_u8  => { shlo_r_32(pvm_ctx, program) },
        199_u8  => { shar_r_32(pvm_ctx, program) },
        200_u8  => { add_64(pvm_ctx, program) },
        201_u8  => { sub_64(pvm_ctx, program) },
        202_u8  => { mul_64(pvm_ctx, program) },
        203_u8  => { div_u_64(pvm_ctx, program) },
        204_u8  => { div_s_64(pvm_ctx, program) },
        205_u8  => { rem_u_64(pvm_ctx, program) },
        206_u8  => { rem_s_64(pvm_ctx, program) },
        207_u8  => { shlo_l_64(pvm_ctx, program) },
        208_u8  => { shlo_r_64(pvm_ctx, program) },
        209_u8  => { shar_r_64(pvm_ctx, program) },
        210_u8  => { and(pvm_ctx, program) },
        211_u8  => { xor(pvm_ctx, program) },
        212_u8  => { or(pvm_ctx, program) },
        213_u8  => { mul_upper_s_s(pvm_ctx, program) },
        214_u8  => { mul_upper_u_u(pvm_ctx, program) },
        215_u8  => { mul_upper_s_u(pvm_ctx, program) },
        216_u8  => { set_lt_u(pvm_ctx, program) },
        217_u8  => { set_lt_s(pvm_ctx, program) },
        218_u8  => { cmov_iz(pvm_ctx, program) },
        219_u8  => { cmov_nz(pvm_ctx, program) },
        220_u8  => { rot_l_64(pvm_ctx, program) },
        221_u8  => { rot_l_32(pvm_ctx, program) },
        222_u8  => { rot_r_64(pvm_ctx, program) },
        223_u8  => { rot_r_32(pvm_ctx, program) },
        224_u8  => { and_inv(pvm_ctx, program) },
        225_u8  => { or_inv(pvm_ctx, program) },
        226_u8  => { xnor(pvm_ctx, program) },
        227_u8  => { max(pvm_ctx, program) },
        228_u8  => { max_u(pvm_ctx, program) },
        229_u8  => { min(pvm_ctx, program) },
        230_u8  => { min_u(pvm_ctx, program) },
        _       => { return ExitReason::panic },   // Panic
    };

    return exit_reason;
}

const TRAP: u8 = 0;
const FALLTHROUGH: u8 = 1;
const ECALLI: u8 = 10;
const LOAD_IMM_64: u8 = 20;
const STORE_IMM_U8: u8 = 30;
const STORE_IMM_U16: u8 = 31;
const STORE_IMM_U32: u8 = 32;
const STORE_IMM_U64: u8 = 33;
const JUMP: u8 = 40;
const JUMP_IND: u8 = 50;
const LOAD_IMM: u8 = 51;
const LOAD_U8: u8 = 52;
const LOAD_I8: u8 = 53;
const LOAD_U16: u8 = 54;
const LOAD_I16: u8 = 55;
const LOAD_U32: u8 = 56;
const LOAD_I32: u8 = 57;
const LOAD_U64: u8 = 58;
const STORE_U8: u8 = 59;
const STORE_U16: u8 = 60;
const STORE_U32: u8 = 61;
const STORE_U64: u8 = 62;
const STORE_IMM_IND_U8: u8 = 70;
const STORE_IMM_IND_U16: u8 = 71;
const STORE_IMM_IND_U32: u8 = 72;
const STORE_IMM_IND_U64: u8 = 73;
const LOAD_IMM_JUMP: u8 = 80;
const BRANCH_EQ_IMM: u8 = 81;
const BRANCH_NE_IMM: u8 = 82;
const BRANCH_LT_U_IMM: u8 = 83;
const BRANCH_LE_U_IMM: u8 = 84;
const BRANCH_GE_U_IMM: u8 = 85;
const BRANCH_GT_U_IMM: u8 = 86;
const BRANCH_LT_S_IMM: u8 = 87;
const BRANCH_LE_S_IMM: u8 = 88;
const BRANCH_GE_S_IMM: u8 = 89;
const BRANCH_GT_S_IMM: u8 = 90;
const MOVE_REG: u8 = 100;
const COUNT_SET_BITS_64: u8 = 102;
const COUNT_SET_BITS_32: u8 = 103;
const LEADING_ZERO_BITS_64: u8 = 104;
const LEADING_ZERO_BITS_32: u8 = 105;
const TRAILING_ZERO_BITS_64: u8 = 106;
const TRAILING_ZERO_BITS_32: u8 = 107;
const SIGN_EXTEND_8: u8 = 108;
const SIGN_EXTEND_16: u8 = 109;
const ZERO_EXTEND_16: u8 = 110;
const REVERSE_BYTES: u8 = 111;
const STORE_IND_U8: u8 = 120;
const STORE_IND_U16: u8 = 121;
const STORE_IND_U32: u8 = 122;
const STORE_IND_U64: u8 = 123;
const LOAD_IND_U8: u8 = 124;
const LOAD_IND_I8: u8 = 125;
const LOAD_IND_U16: u8 = 126;
const LOAD_IND_I16: u8 = 127;
const LOAD_IND_U32: u8 = 128;
const LOAD_IND_I32: u8 = 129;
const LOAD_IND_U64: u8 = 130;
const ADD_IMM_32: u8 = 131;
const AND_IMM: u8 = 132;
const XOR_IMM: u8 = 133;
const OR_IMM: u8 = 134;
const MUL_IMM_32: u8 = 135;
const SET_LT_U_IMM: u8 = 136;
const SET_LT_S_IMM: u8 = 137;
const SHLO_L_IMM_32: u8 = 138;
const SHLO_R_IMM_32: u8 = 139;
const SHAR_R_IMM_32: u8 = 140;
const NEG_ADD_IMM_32: u8 = 141;
const SET_GT_U_IMM: u8 = 142;
const SET_GT_S_IMM: u8 = 143;
const SHLO_L_IMM_ALT_32: u8 = 144;
const SHLO_R_IMM_ALT_32: u8 = 145;
const SHAR_R_IMM_ALT_32: u8 = 146;
const CMOV_IZ_IMM: u8 = 147;
const ADD_IMM_64: u8 = 149;
const MUL_IMM_64: u8 = 150;
const SHLO_L_IMM_64: u8 = 151;
const SHLO_R_IMM_64: u8 = 152;
const SHAR_R_IMM_64: u8 = 153;
const NEG_ADD_IMM_64: u8 = 154;
const SHLO_L_IMM_ALT_64: u8 = 155;
const SHLO_R_IMM_ALT_64: u8 = 156;
const SHAR_R_IMM_ALT_64: u8 = 157;
const ROT_R_64_IMM: u8 = 158;
const ROT_R_64_IMM_ALT: u8 = 159;
const ROT_R_32_IMM: u8 = 160;
const ROT_R_32_IMM_ALT: u8 = 161;
const BRANCH_EQ: u8 = 170;
const BRANCH_NE: u8 = 171;
const BRANCH_LT_U: u8 = 172;
const BRANCH_LT_S: u8 = 173;
const BRANCH_GE_U: u8 = 174;
const BRANCH_GE_S: u8 = 175;
const LOAD_IMM_JUMP_IND: u8 = 180;
const ADD_32: u8 = 190;
const SUB_32: u8 = 191;
const MUL_32: u8 = 192;
const DIV_U_32: u8 = 193;
const DIV_S_32: u8 = 194;
const REM_U_32: u8 = 195;
const REM_S_32: u8 = 196;
const SHLO_L_32: u8 = 197;
const SHLO_R_32: u8 = 198;
const SHAR_R_32: u8 = 199;
const ADD_64: u8 = 200;
const SUB_64: u8 = 201;
const MUL_64: u8 = 202;
const DIV_U_64: u8 = 203;
const DIV_S_64: u8 = 204;
const REM_U_64: u8 = 205;
const REM_S_64: u8 = 206;
const SHLO_L_64: u8 = 207;
const SHLO_R_64: u8 = 208;
const SHAR_R_64: u8 = 209;
const AND: u8 = 210;
const XOR: u8 = 211;
const OR: u8 = 212;
const MUL_UPPER_S_S: u8 = 213;
const MUL_UPPER_U_U: u8 = 214;
const MUL_UPPER_S_U: u8 = 215;
const SET_LT_U: u8 = 216;
const SET_LT_S: u8 = 217;
const CMOV_IZ: u8 = 218;
const CMOV_NZ: u8 = 219;
const ROT_L_64: u8 = 220;
const ROT_L_32: u8 = 221;
const ROT_R_64: u8 = 222;
const ROT_R_32: u8 = 223;
const AND_INV: u8 = 224;
const OR_INV: u8 = 225;
const XNOR: u8 = 226;
const MAX: u8 = 227;
const MAX_U: u8 = 228;
const MIN: u8 = 229;
const MIN_U: u8 = 230;
