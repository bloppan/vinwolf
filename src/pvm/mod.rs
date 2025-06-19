use crate::types::{Context, ExitReason, Program};
use crate::utils::codec::{BytesReader, Decode};

use isa::one_offset::*;
use isa::no_arg::*;
use isa::one_imm::*;
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
    //println!("Invoke PVM");
    let program = Program::decode(&mut BytesReader::new(program_blob)).unwrap(); // TODO handle error
    //let mut step = 1;
    
    loop {
        
        let exit_reason = single_step_pvm(pvm_ctx, &program);
        //println!("exit reason = {:?}", exit_reason);
        
        match exit_reason {
            ExitReason::Continue => {
                // continue
            },
            ExitReason::OutOfGas => {
                return ExitReason::OutOfGas;
            },
            // TODO arreglar esto
            /*ExitReason::panic |*/ ExitReason::Halt => {
                //println!("step = {step}, pc = {:?}, opcode = {:?} \t, reg = {:?}", pvm_ctx.pc.clone(), program.code[pvm_ctx.pc.clone() as usize], pvm_ctx.reg);
                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien)
                return ExitReason::halt;
            },
            _ => { 
                //println!("step = {step}, pc = {:?}, opcode = {:?} \t, reg = {:?}", pvm_ctx.pc.clone(), program.code[pvm_ctx.pc.clone() as usize], pvm_ctx.reg);
                //println!("")
                return exit_reason; 
            },
        } 
        //println!("step = {step}, pc = {:?}, opcode = {:?} \t, reg = {:?}", pvm_ctx.pc.clone(), program.code[pc_copy as usize], pvm_ctx.reg);
        //step += 1;   
    }
}


fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    pvm_ctx.gas -= 1;

    if pvm_ctx.gas < 0 {
        return ExitReason::OutOfGas;
    }

    let exit_reason = match program.code[pvm_ctx.pc as usize] { 

        TRAP                    => { trap() },
        FALLTHROUGH             => { fallthrough(pvm_ctx, program) },
        ECALLI                  => { ecalli(pvm_ctx, program) },
        LOAD_IMM_64             => { load_imm_64(pvm_ctx, program) },
        STORE_IMM_U8            => { store_imm_u8(pvm_ctx, program) },
        STORE_IMM_U16           => { store_imm_u16(pvm_ctx, program) },
        STORE_IMM_U32           => { store_imm_u32(pvm_ctx, program) },
        STORE_IMM_U64           => { store_imm_u64(pvm_ctx, program) },
        JUMP                    => { jump(pvm_ctx, program) },
        JUMP_IND                => { jump_ind(pvm_ctx, program) },
        LOAD_IMM                => { load_imm(pvm_ctx, program) },
        LOAD_U8                 => { load_u8(pvm_ctx, program) },
        LOAD_I8                 => { load_i8(pvm_ctx, program) },
        LOAD_U16                => { load_u16(pvm_ctx, program) },
        LOAD_I16                => { load_i16(pvm_ctx, program) },
        LOAD_U32                => { load_u32(pvm_ctx, program) },
        LOAD_I32                => { load_i32(pvm_ctx, program) },
        LOAD_U64                => { load_u64(pvm_ctx, program) },
        STORE_U8                => { store_u8(pvm_ctx, program) },
        STORE_U16               => { store_u16(pvm_ctx, program) },
        STORE_U32               => { store_u32(pvm_ctx, program) },
        STORE_U64               => { store_u64(pvm_ctx, program) },
        STORE_IMM_IND_U8        => { store_imm_ind_u8(pvm_ctx, program) },
        STORE_IMM_IND_U16       => { store_imm_ind_u16(pvm_ctx, program) },
        STORE_IMM_IND_U32       => { store_imm_ind_u32(pvm_ctx, program) },
        STORE_IMM_IND_U64       => { store_imm_ind_u64(pvm_ctx, program) },
        LOAD_IMM_JUMP           => { load_imm_jump(pvm_ctx, program) },
        BRANCH_EQ_IMM           => { branch_eq_imm(pvm_ctx, program) },
        BRANCH_NE_IMM           => { branch_ne_imm(pvm_ctx, program) },
        BRANCH_LT_U_IMM         => { branch_lt_u_imm(pvm_ctx, program) },
        BRANCH_LE_U_IMM         => { branch_le_u_imm(pvm_ctx, program) },
        BRANCH_GE_U_IMM         => { branch_ge_u_imm(pvm_ctx, program) },
        BRANCH_GT_U_IMM         => { branch_gt_u_imm(pvm_ctx, program) },
        BRANCH_LT_S_IMM         => { branch_lt_s_imm(pvm_ctx, program) },
        BRANCH_LE_S_IMM         => { branch_le_s_imm(pvm_ctx, program) },
        BRANCH_GE_S_IMM         => { branch_ge_s_imm(pvm_ctx, program) },
        BRANCH_GT_S_IMM         => { branch_gt_s_imm(pvm_ctx, program) },
        MOVE_REG                => { move_reg(pvm_ctx, program) }, 
        SBRK                    => { sbrk(pvm_ctx, program) },
        COUNT_SET_BITS_64       => { count_set_bits_64(pvm_ctx, program) },
        COUNT_SET_BITS_32       => { count_set_bits_32(pvm_ctx, program) },
        LEADING_ZERO_BITS_64    => { leading_zero_bits_64(pvm_ctx, program) },
        LEADING_ZERO_BITS_32    => { leading_zero_bits_32(pvm_ctx, program) },
        TRAILING_ZERO_BITS_64   => { trailing_zero_bits_64(pvm_ctx, program) },
        TRAILING_ZERO_BITS_32   => { trailing_zero_bits_32(pvm_ctx, program) },
        SIGN_EXTEND_8           => { sign_extend_8(pvm_ctx, program) },
        SIGN_EXTEND_16          => { sign_extend_16(pvm_ctx, program) },
        ZERO_EXTEND_16          => { zero_extend_16(pvm_ctx, program) },
        REVERSE_BYTES           => { reverse_bytes(pvm_ctx, program) },
        STORE_IND_U8            => { store_ind_u8(pvm_ctx, program) },
        STORE_IND_U16           => { store_ind_u16(pvm_ctx, program) },
        STORE_IND_U32           => { store_ind_u32(pvm_ctx, program) },
        STORE_IND_U64           => { store_ind_u64(pvm_ctx, program) },
        LOAD_IND_U8             => { load_ind_u8(pvm_ctx, program) },
        LOAD_IND_I8             => { load_ind_i8(pvm_ctx, program) },
        LOAD_IND_U16            => { load_ind_u16(pvm_ctx, program) },
        LOAD_IND_I16            => { load_ind_i16(pvm_ctx, program) },
        LOAD_IND_U32            => { load_ind_u32(pvm_ctx, program) },
        LOAD_IND_I32            => { load_ind_i32(pvm_ctx, program) },
        LOAD_IND_U64            => { load_ind_u64(pvm_ctx, program) },
        ADD_IMM_32              => { add_imm_32(pvm_ctx, program) }, 
        AND_IMM                 => { and_imm(pvm_ctx, program) },
        XOR_IMM                 => { xor_imm(pvm_ctx, program) },
        OR_IMM                  => { or_imm(pvm_ctx, program) },
        MUL_IMM_32              => { mul_imm_32(pvm_ctx, program) },
        SET_LT_U_IMM            => { set_lt_u_imm(pvm_ctx, program) },
        SET_LT_S_IMM            => { set_lt_s_imm(pvm_ctx, program) },
        SHLO_L_IMM_32           => { shlo_l_imm_32(pvm_ctx, program) },
        SHLO_R_IMM_32           => { shlo_r_imm_32(pvm_ctx, program) },
        SHAR_R_IMM_32           => { shar_r_imm_32(pvm_ctx, program) },
        NEG_ADD_IMM_32          => { neg_add_imm_32(pvm_ctx, program) },
        SET_GT_U_IMM            => { set_gt_u_imm(pvm_ctx, program) },
        SET_GT_S_IMM            => { set_gt_s_imm(pvm_ctx, program) },
        SHLO_L_IMM_ALT_32       => { shlo_l_imm_alt_32(pvm_ctx, program) },
        SHLO_R_IMM_ALT_32       => { shlo_r_imm_alt_32(pvm_ctx, program) },
        SHAR_R_IMM_ALT_32       => { shar_r_imm_alt_32(pvm_ctx, program) },
        CMOV_IZ_IMM             => { cmov_iz_imm(pvm_ctx, program) },
        CMOV_NZ_IMM             => { cmov_nz_imm(pvm_ctx, program) },
        ADD_IMM_64              => { add_imm_64(pvm_ctx, program) },
        MUL_IMM_64              => { mul_imm_64(pvm_ctx, program) },
        SHLO_L_IMM_64           => { shlo_l_imm_64(pvm_ctx, program) },
        SHLO_R_IMM_64           => { shlo_r_imm_64(pvm_ctx, program) },
        SHAR_R_IMM_64           => { shar_r_imm_64(pvm_ctx, program) },    
        NEG_ADD_IMM_64          => { neg_add_imm_64(pvm_ctx, program) },
        SHLO_L_IMM_ALT_64       => { shlo_l_imm_alt_64(pvm_ctx, program) },
        SHLO_R_IMM_ALT_64       => { shlo_r_imm_alt_64(pvm_ctx, program) },
        SHAR_R_IMM_ALT_64       => { shar_r_imm_alt_64(pvm_ctx, program) },
        ROT_R_64_IMM            => { rot_r_64_imm(pvm_ctx, program) },
        ROT_R_64_IMM_ALT        => { rot_r_64_imm_alt(pvm_ctx, program) },
        ROT_R_32_IMM            => { rot_r_32_imm(pvm_ctx, program) },
        ROT_R_32_IMM_ALT        => { rot_r_32_imm_alt(pvm_ctx, program) },
        BRANCH_EQ               => { branch_eq(pvm_ctx, program) },
        BRANCH_NE               => { branch_ne(pvm_ctx, program) },
        BRANCH_LT_U             => { branch_lt_u(pvm_ctx, program) },
        BRANCH_LT_S             => { branch_lt_s(pvm_ctx, program) },
        BRANCH_GE_U             => { branch_ge_u(pvm_ctx, program) },
        BRANCH_GE_S             => { branch_ge_s(pvm_ctx, program) },
        LOAD_IMM_JUMP_IND       => { load_imm_jump_ind(pvm_ctx, program) },
        ADD_32                  => { add_32(pvm_ctx, program) },
        SUB_32                  => { sub_32(pvm_ctx, program) },
        MUL_32                  => { mul_32(pvm_ctx, program) },
        DIV_U_32                => { div_u_32(pvm_ctx, program) },
        DIV_S_32                => { div_s_32(pvm_ctx, program) },
        REM_U_32                => { rem_u_32(pvm_ctx, program) },
        REM_S_32                => { rem_s_32(pvm_ctx, program) },
        SHLO_L_32               => { shlo_l_32(pvm_ctx, program) },
        SHLO_R_32               => { shlo_r_32(pvm_ctx, program) },
        SHAR_R_32               => { shar_r_32(pvm_ctx, program) },
        ADD_64                  => { add_64(pvm_ctx, program) },
        SUB_64                  => { sub_64(pvm_ctx, program) },
        MUL_64                  => { mul_64(pvm_ctx, program) },
        DIV_U_64                => { div_u_64(pvm_ctx, program) },
        DIV_S_64                => { div_s_64(pvm_ctx, program) },
        REM_U_64                => { rem_u_64(pvm_ctx, program) },
        REM_S_64                => { rem_s_64(pvm_ctx, program) },
        SHLO_L_64               => { shlo_l_64(pvm_ctx, program) },
        SHLO_R_64               => { shlo_r_64(pvm_ctx, program) },
        SHAR_R_64               => { shar_r_64(pvm_ctx, program) },
        AND                     => { and(pvm_ctx, program) },
        XOR                     => { xor(pvm_ctx, program) },
        OR                      => { or(pvm_ctx, program) },
        MUL_UPPER_S_S           => { mul_upper_s_s(pvm_ctx, program) },
        MUL_UPPER_U_U           => { mul_upper_u_u(pvm_ctx, program) },
        MUL_UPPER_S_U           => { mul_upper_s_u(pvm_ctx, program) },
        SET_LT_U                => { set_lt_u(pvm_ctx, program) },
        SET_LT_S                => { set_lt_s(pvm_ctx, program) },
        CMOV_IZ                 => { cmov_iz(pvm_ctx, program) },
        CMOV_NZ                 => { cmov_nz(pvm_ctx, program) },
        ROT_L_64                => { rot_l_64(pvm_ctx, program) },
        ROT_L_32                => { rot_l_32(pvm_ctx, program) },
        ROT_R_64                => { rot_r_64(pvm_ctx, program) },
        ROT_R_32                => { rot_r_32(pvm_ctx, program) },
        AND_INV                 => { and_inv(pvm_ctx, program) },
        OR_INV                  => { or_inv(pvm_ctx, program) },
        XNOR                    => { xnor(pvm_ctx, program) },
        MAX                     => { max(pvm_ctx, program) },
        MAX_U                   => { max_u(pvm_ctx, program) },
        MIN                     => { min(pvm_ctx, program) },
        MIN_U                   => { min_u(pvm_ctx, program) },
        _                       => { println!("Unknown instruction!"); return ExitReason::panic },
    };
    //println!("pc = {:?}, opcode = {:?}, reg = {:?}", pvm_ctx.pc, program.code[pvm_ctx.pc as usize], pvm_ctx.reg);
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
const SBRK: u8 = 101;
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
const CMOV_NZ_IMM: u8 = 148;
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
