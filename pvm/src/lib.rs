pub mod isa;
pub mod mm;
pub mod hostcall;
pub mod pvm_types;

use std::sync::{Mutex, LazyLock};
use codec::{BytesReader, Decode};
use crate::pvm_types::{Gas, RegSize, RamMemory, Registers};
use crate::pvm_types::{Context, ExitReason, Program};
use utils::log;

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
use constants::pvm::*;

pub fn invoke_pvm(program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {

    log::debug!("Invoke inner pvm");

    loop {
        
        let exit_reason = single_step_pvm(&program, pc, gas, ram, reg);

        match exit_reason {
            ExitReason::Continue => {
                continue;
            },
            ExitReason::OutOfGas => {
                log::error!("PVM: Out of gas!");
                return ExitReason::OutOfGas;
            },
            // TODO arreglar esto
            /*ExitReason::panic |*/ ExitReason::Halt => {
                log::debug!("PVM: Halt");
                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien) TODO
                return ExitReason::halt;
            },
            _ => { 
                log::debug!("PVM: {:?}", exit_reason);
                return exit_reason; 
            },
        }
       
    }
}


static INDEX: LazyLock<Mutex<u32>> = LazyLock::new(|| {Mutex::new(0)});

fn single_step_pvm(program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {

    *gas -= 1;

    if *gas < 0 {
        return ExitReason::OutOfGas;
    }

    let exit_reason = match program.code[*pc as usize] { 

        TRAP                    => { trap(program, pc, gas, ram, reg) },
        FALLTHROUGH             => { fallthrough(program, pc, gas, ram, reg) },
        ECALLI                  => { ecalli(program, pc, gas, ram, reg) },
        LOAD_IMM_64             => { load_imm_64(program, pc, gas, ram, reg) },
        STORE_IMM_U8            => { store_imm_u8(program, pc, gas, ram, reg) },
        STORE_IMM_U16           => { store_imm_u16(program, pc, gas, ram, reg) },
        STORE_IMM_U32           => { store_imm_u32(program, pc, gas, ram, reg) },
        STORE_IMM_U64           => { store_imm_u64(program, pc, gas, ram, reg) },
        JUMP                    => { jump(program, pc, gas, ram, reg) },
        JUMP_IND                => { jump_ind(program, pc, gas, ram, reg) },
        LOAD_IMM                => { load_imm(program, pc, gas, ram, reg) },
        LOAD_U8                 => { load_u8(program, pc, gas, ram, reg) },
        LOAD_I8                 => { load_i8(program, pc, gas, ram, reg) },
        LOAD_U16                => { load_u16(program, pc, gas, ram, reg) },
        LOAD_I16                => { load_i16(program, pc, gas, ram, reg) },
        LOAD_U32                => { load_u32(program, pc, gas, ram, reg) },
        LOAD_I32                => { load_i32(program, pc, gas, ram, reg) },
        LOAD_U64                => { load_u64(program, pc, gas, ram, reg) },
        STORE_U8                => { store_u8(program, pc, gas, ram, reg) },
        STORE_U16               => { store_u16(program, pc, gas, ram, reg) },
        STORE_U32               => { store_u32(program, pc, gas, ram, reg) },
        STORE_U64               => { store_u64(program, pc, gas, ram, reg) },
        STORE_IMM_IND_U8        => { store_imm_ind_u8(program, pc, gas, ram, reg) },
        STORE_IMM_IND_U16       => { store_imm_ind_u16(program, pc, gas, ram, reg) },
        STORE_IMM_IND_U32       => { store_imm_ind_u32(program, pc, gas, ram, reg) },
        STORE_IMM_IND_U64       => { store_imm_ind_u64(program, pc, gas, ram, reg) },
        LOAD_IMM_JUMP           => { load_imm_jump(program, pc, gas, ram, reg) },
        BRANCH_EQ_IMM           => { branch_eq_imm(program, pc, gas, ram, reg) },
        BRANCH_NE_IMM           => { branch_ne_imm(program, pc, gas, ram, reg) },
        BRANCH_LT_U_IMM         => { branch_lt_u_imm(program, pc, gas, ram, reg) },
        BRANCH_LE_U_IMM         => { branch_le_u_imm(program, pc, gas, ram, reg) },
        BRANCH_GE_U_IMM         => { branch_ge_u_imm(program, pc, gas, ram, reg) },
        BRANCH_GT_U_IMM         => { branch_gt_u_imm(program, pc, gas, ram, reg) },
        BRANCH_LT_S_IMM         => { branch_lt_s_imm(program, pc, gas, ram, reg) },
        BRANCH_LE_S_IMM         => { branch_le_s_imm(program, pc, gas, ram, reg) },
        BRANCH_GE_S_IMM         => { branch_ge_s_imm(program, pc, gas, ram, reg) },
        BRANCH_GT_S_IMM         => { branch_gt_s_imm(program, pc, gas, ram, reg) },
        MOVE_REG                => { move_reg(program, pc, gas, ram, reg) }, 
        SBRK                    => { sbrk(program, pc, gas, ram, reg) },
        COUNT_SET_BITS_64       => { count_set_bits_64(program, pc, gas, ram, reg) },
        COUNT_SET_BITS_32       => { count_set_bits_32(program, pc, gas, ram, reg) },
        LEADING_ZERO_BITS_64    => { leading_zero_bits_64(program, pc, gas, ram, reg) },
        LEADING_ZERO_BITS_32    => { leading_zero_bits_32(program, pc, gas, ram, reg) },
        TRAILING_ZERO_BITS_64   => { trailing_zero_bits_64(program, pc, gas, ram, reg) },
        TRAILING_ZERO_BITS_32   => { trailing_zero_bits_32(program, pc, gas, ram, reg) },
        SIGN_EXTEND_8           => { sign_extend_8(program, pc, gas, ram, reg) },
        SIGN_EXTEND_16          => { sign_extend_16(program, pc, gas, ram, reg) },
        ZERO_EXTEND_16          => { zero_extend_16(program, pc, gas, ram, reg) },
        REVERSE_BYTES           => { reverse_bytes(program, pc, gas, ram, reg) },
        STORE_IND_U8            => { store_ind_u8(program, pc, gas, ram, reg) },
        STORE_IND_U16           => { store_ind_u16(program, pc, gas, ram, reg) },
        STORE_IND_U32           => { store_ind_u32(program, pc, gas, ram, reg) },
        STORE_IND_U64           => { store_ind_u64(program, pc, gas, ram, reg) },
        LOAD_IND_U8             => { load_ind_u8(program, pc, gas, ram, reg) },
        LOAD_IND_I8             => { load_ind_i8(program, pc, gas, ram, reg) },
        LOAD_IND_U16            => { load_ind_u16(program, pc, gas, ram, reg) },
        LOAD_IND_I16            => { load_ind_i16(program, pc, gas, ram, reg) },
        LOAD_IND_U32            => { load_ind_u32(program, pc, gas, ram, reg) },
        LOAD_IND_I32            => { load_ind_i32(program, pc, gas, ram, reg) },
        LOAD_IND_U64            => { load_ind_u64(program, pc, gas, ram, reg) },
        ADD_IMM_32              => { add_imm_32(program, pc, gas, ram, reg) }, 
        AND_IMM                 => { and_imm(program, pc, gas, ram, reg) },
        XOR_IMM                 => { xor_imm(program, pc, gas, ram, reg) },
        OR_IMM                  => { or_imm(program, pc, gas, ram, reg) },
        MUL_IMM_32              => { mul_imm_32(program, pc, gas, ram, reg) },
        SET_LT_U_IMM            => { set_lt_u_imm(program, pc, gas, ram, reg) },
        SET_LT_S_IMM            => { set_lt_s_imm(program, pc, gas, ram, reg) },
        SHLO_L_IMM_32           => { shlo_l_imm_32(program, pc, gas, ram, reg) },
        SHLO_R_IMM_32           => { shlo_r_imm_32(program, pc, gas, ram, reg) },
        SHAR_R_IMM_32           => { shar_r_imm_32(program, pc, gas, ram, reg) },
        NEG_ADD_IMM_32          => { neg_add_imm_32(program, pc, gas, ram, reg) },
        SET_GT_U_IMM            => { set_gt_u_imm(program, pc, gas, ram, reg) },
        SET_GT_S_IMM            => { set_gt_s_imm(program, pc, gas, ram, reg) },
        SHLO_L_IMM_ALT_32       => { shlo_l_imm_alt_32(program, pc, gas, ram, reg) },
        SHLO_R_IMM_ALT_32       => { shlo_r_imm_alt_32(program, pc, gas, ram, reg) },
        SHAR_R_IMM_ALT_32       => { shar_r_imm_alt_32(program, pc, gas, ram, reg) },
        CMOV_IZ_IMM             => { cmov_iz_imm(program, pc, gas, ram, reg) },
        CMOV_NZ_IMM             => { cmov_nz_imm(program, pc, gas, ram, reg) },
        ADD_IMM_64              => { add_imm_64(program, pc, gas, ram, reg) },
        MUL_IMM_64              => { mul_imm_64(program, pc, gas, ram, reg) },
        SHLO_L_IMM_64           => { shlo_l_imm_64(program, pc, gas, ram, reg) },
        SHLO_R_IMM_64           => { shlo_r_imm_64(program, pc, gas, ram, reg) },
        SHAR_R_IMM_64           => { shar_r_imm_64(program, pc, gas, ram, reg) },    
        NEG_ADD_IMM_64          => { neg_add_imm_64(program, pc, gas, ram, reg) },
        SHLO_L_IMM_ALT_64       => { shlo_l_imm_alt_64(program, pc, gas, ram, reg) },
        SHLO_R_IMM_ALT_64       => { shlo_r_imm_alt_64(program, pc, gas, ram, reg) },
        SHAR_R_IMM_ALT_64       => { shar_r_imm_alt_64(program, pc, gas, ram, reg) },
        ROT_R_64_IMM            => { rot_r_64_imm(program, pc, gas, ram, reg) },
        ROT_R_64_IMM_ALT        => { rot_r_64_imm_alt(program, pc, gas, ram, reg) },
        ROT_R_32_IMM            => { rot_r_32_imm(program, pc, gas, ram, reg) },
        ROT_R_32_IMM_ALT        => { rot_r_32_imm_alt(program, pc, gas, ram, reg) },
        BRANCH_EQ               => { branch_eq(program, pc, gas, ram, reg) },
        BRANCH_NE               => { branch_ne(program, pc, gas, ram, reg) },
        BRANCH_LT_U             => { branch_lt_u(program, pc, gas, ram, reg) },
        BRANCH_LT_S             => { branch_lt_s(program, pc, gas, ram, reg) },
        BRANCH_GE_U             => { branch_ge_u(program, pc, gas, ram, reg) },
        BRANCH_GE_S             => { branch_ge_s(program, pc, gas, ram, reg) },
        LOAD_IMM_JUMP_IND       => { load_imm_jump_ind(program, pc, gas, ram, reg) },
        ADD_32                  => { add_32(program, pc, gas, ram, reg) },
        SUB_32                  => { sub_32(program, pc, gas, ram, reg) },
        MUL_32                  => { mul_32(program, pc, gas, ram, reg) },
        DIV_U_32                => { div_u_32(program, pc, gas, ram, reg) },
        DIV_S_32                => { div_s_32(program, pc, gas, ram, reg) },
        REM_U_32                => { rem_u_32(program, pc, gas, ram, reg) },
        REM_S_32                => { rem_s_32(program, pc, gas, ram, reg) },
        SHLO_L_32               => { shlo_l_32(program, pc, gas, ram, reg) },
        SHLO_R_32               => { shlo_r_32(program, pc, gas, ram, reg) },
        SHAR_R_32               => { shar_r_32(program, pc, gas, ram, reg) },
        ADD_64                  => { add_64(program, pc, gas, ram, reg) },
        SUB_64                  => { sub_64(program, pc, gas, ram, reg) },
        MUL_64                  => { mul_64(program, pc, gas, ram, reg) },
        DIV_U_64                => { div_u_64(program, pc, gas, ram, reg) },
        DIV_S_64                => { div_s_64(program, pc, gas, ram, reg) },
        REM_U_64                => { rem_u_64(program, pc, gas, ram, reg) },
        REM_S_64                => { rem_s_64(program, pc, gas, ram, reg) },
        SHLO_L_64               => { shlo_l_64(program, pc, gas, ram, reg) },
        SHLO_R_64               => { shlo_r_64(program, pc, gas, ram, reg) },
        SHAR_R_64               => { shar_r_64(program, pc, gas, ram, reg) },
        AND                     => { and(program, pc, gas, ram, reg) },
        XOR                     => { xor(program, pc, gas, ram, reg) },
        OR                      => { or(program, pc, gas, ram, reg) },
        MUL_UPPER_S_S           => { mul_upper_s_s(program, pc, gas, ram, reg) },
        MUL_UPPER_U_U           => { mul_upper_u_u(program, pc, gas, ram, reg) },
        MUL_UPPER_S_U           => { mul_upper_s_u(program, pc, gas, ram, reg) },
        SET_LT_U                => { set_lt_u(program, pc, gas, ram, reg) },
        SET_LT_S                => { set_lt_s(program, pc, gas, ram, reg) },
        CMOV_IZ                 => { cmov_iz(program, pc, gas, ram, reg) },
        CMOV_NZ                 => { cmov_nz(program, pc, gas, ram, reg) },
        ROT_L_64                => { rot_l_64(program, pc, gas, ram, reg) },
        ROT_L_32                => { rot_l_32(program, pc, gas, ram, reg) },
        ROT_R_64                => { rot_r_64(program, pc, gas, ram, reg) },
        ROT_R_32                => { rot_r_32(program, pc, gas, ram, reg) },
        AND_INV                 => { and_inv(program, pc, gas, ram, reg) },
        OR_INV                  => { or_inv(program, pc, gas, ram, reg) },
        XNOR                    => { xnor(program, pc, gas, ram, reg) },
        MAX                     => { max(program, pc, gas, ram, reg) },
        MAX_U                   => { max_u(program, pc, gas, ram, reg) },
        MIN                     => { min(program, pc, gas, ram, reg) },
        MIN_U                   => { min_u(program, pc, gas, ram, reg) },
        _                       => { println!("Unknown instruction!"); return ExitReason::panic },
    };

    //log::trace!("pc = {:?}, opcode = {:03?}, index = {:05?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, INDEX.lock().unwrap(), 20_000_000_i64.saturating_sub(pvm_ctx.gas), pvm_ctx.reg);
    //*INDEX.lock().unwrap() += 1;

    //println!("pc = {:?}, opcode = {:?}, reg = {:?}", pvm_ctx.pc, program.code[pvm_ctx.pc as usize], pvm_ctx.reg);
    return exit_reason;
}
