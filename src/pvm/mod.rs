pub mod isa;
pub mod mm;
pub mod hostcall;
pub mod pvm_constants;

use crate::jam_types::{Context, ExitReason, Program};
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
use pvm_constants::*;

pub fn invoke_pvm(pvm_ctx: &mut Context, program_blob: &[u8]) -> ExitReason {

    log::debug!("Invoke inner pvm");

    let program = match Program::decode(&mut BytesReader::new(program_blob)) {
        Ok(program) => { program },
        Err(_) => { 
            log::error!("Panic: Decoding program");
            return ExitReason::panic; 
        }
    };

    let mut opcode_copy = program.code[pvm_ctx.pc.clone() as usize];

    loop {
        
        let exit_reason = single_step_pvm(pvm_ctx, &program);
        
        match exit_reason {
            ExitReason::Continue => {
                // continue
            },
            ExitReason::OutOfGas => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::error!("PVM: Out of gas!");
                return ExitReason::OutOfGas;
            },
            // TODO arreglar esto
            /*ExitReason::panic |*/ ExitReason::Halt => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::debug!("PVM: Halt");
                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien) TODO
                return ExitReason::halt;
            },
            _ => { 
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::debug!("PVM: {:?}", exit_reason);
                return exit_reason; 
            },
        }
       
        log::trace!("pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
        opcode_copy = program.code[pvm_ctx.pc.clone() as usize];
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

