pub mod isa;
pub mod mm;
pub mod hostcall;
pub mod pvm_types;

use std::sync::{Mutex, LazyLock};
use codec::{BytesReader, Decode};
use crate::pvm_types::{Gas, RegSize, RamMemory, Registers};
use crate::pvm_types::{RamAddress, ExitReason, Program};
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

const PAGE_SHIFT: RamAddress = PAGE_SIZE.trailing_zeros();
const PAGE_MASK: RamAddress = PAGE_SIZE - 1;
const BOUNDS_MASK: RamAddress = NUM_PAGES - 1;

#[macro_export] macro_rules! page_index {
    ($addr:expr) => {
        $addr >> crate::PAGE_SHIFT
    };
}

#[macro_export] macro_rules! page_offset {
    ($addr:expr) => {
        $addr & crate::PAGE_MASK
    };
}

#[macro_export] macro_rules! mem_bounds {
    ($addr:expr) => {
        $addr & crate::BOUNDS_MASK
    };
}


/*#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub pc: RegSize,
    pub gas: Gas,
    pub ram: RamMemory,
    pub reg: Registers,
    pub page_fault: Option<RamAddress>,
}*/

pub fn invoke_pvm(program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {

    log::debug!("Invoke inner pvm");

    /*let program = match Program::decode(&mut BytesReader::new(program_blob)) {
        Ok(program) => { program },
        Err(_) => { 
            log::error!("Panic: Decoding program");
            return ExitReason::panic; 
        }
    };*/

    let mut opcode_copy = program.code[pc.clone() as usize];

    loop {
        
        *gas -= 1;

        if *gas < 0 {
            return ExitReason::OutOfGas;
        }
        
        let exit_reason = dispatch_instr(program.code[*pc as usize], program, pc, gas, ram, reg);

        //let exit_reason = single_step_pvm(&program, pc, gas, ram, reg);

        match exit_reason {
            ExitReason::Continue => {
                log::trace!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pc.clone(), opcode_copy, gas, reg);
                continue;
            },
            ExitReason::OutOfGas => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pc.clone(), opcode_copy, gas, reg);
                log::error!("PVM: Out of gas!");
                return ExitReason::OutOfGas;
            },
            // TODO arreglar esto
            /*ExitReason::panic |*/ ExitReason::Halt => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pc.clone(), opcode_copy, gas, reg);
                log::debug!("PVM: Halt");

                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien) TODO
                return ExitReason::halt;
            },
            _ => { 
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pc.clone(), opcode_copy, gas, reg);
                log::debug!("PVM: {:?}", exit_reason);
                return exit_reason; 
            },
        }
       
    }
}

type FunctionPtr = fn(&Program, &mut RegSize, &mut Gas, &mut RamMemory, &mut Registers) -> ExitReason;

macro_rules! init_function_array {
    ($($index:expr => $func:ident),* $(,)?) => {{
        let mut arr = [trap as FunctionPtr; 256];
        $(
            arr[$index as usize] = $func;
        )*
        arr
    }};
}

const FUNCTION_ARRAY: [FunctionPtr; 256] = init_function_array! {
    TRAP => trap,
    FALLTHROUGH => fallthrough,
    ECALLI => ecalli,
    LOAD_IMM_64 => load_imm_64,
    STORE_IMM_U8 => store_imm_u8,
    STORE_IMM_U16 => store_imm_u16,
    STORE_IMM_U32 => store_imm_u32,
    STORE_IMM_U64 => store_imm_u64,
    JUMP => jump,
    JUMP_IND => jump_ind,
    LOAD_IMM => load_imm,
    LOAD_U8 => load_u8,
    LOAD_I8 => load_i8,
    LOAD_U16 => load_u16,
    LOAD_I16 => load_i16,
    LOAD_U32 => load_u32,
    LOAD_I32 => load_i32,
    LOAD_U64 => load_u64,
    STORE_U8 => store_u8,
    STORE_U16 => store_u16,
    STORE_U32 => store_u32,
    STORE_U64 => store_u64,
    STORE_IMM_IND_U8 => store_imm_ind_u8,
    STORE_IMM_IND_U16 => store_imm_ind_u16,
    STORE_IMM_IND_U32 => store_imm_ind_u32,
    STORE_IMM_IND_U64 => store_imm_ind_u64,
    LOAD_IMM_JUMP => load_imm_jump,
    BRANCH_EQ_IMM => branch_eq_imm,
    BRANCH_NE_IMM => branch_ne_imm,
    BRANCH_LT_U_IMM => branch_lt_u_imm,
    BRANCH_LE_U_IMM => branch_le_u_imm,
    BRANCH_GE_U_IMM => branch_ge_u_imm,
    BRANCH_GT_U_IMM => branch_gt_u_imm,
    BRANCH_LT_S_IMM => branch_lt_s_imm,
    BRANCH_LE_S_IMM => branch_le_s_imm,
    BRANCH_GE_S_IMM => branch_ge_s_imm,
    BRANCH_GT_S_IMM => branch_gt_s_imm,
    MOVE_REG => move_reg,
    SBRK => sbrk,
    COUNT_SET_BITS_64 => count_set_bits_64,
    COUNT_SET_BITS_32 => count_set_bits_32,
    LEADING_ZERO_BITS_64 => leading_zero_bits_64,
    LEADING_ZERO_BITS_32 => leading_zero_bits_32,
    TRAILING_ZERO_BITS_64 => trailing_zero_bits_64,
    TRAILING_ZERO_BITS_32 => trailing_zero_bits_32,
    SIGN_EXTEND_8 => sign_extend_8,
    SIGN_EXTEND_16 => sign_extend_16,
    ZERO_EXTEND_16 => zero_extend_16,
    REVERSE_BYTES => reverse_bytes,
    STORE_IND_U8 => store_ind_u8,
    STORE_IND_U16 => store_ind_u16,
    STORE_IND_U32 => store_ind_u32,
    STORE_IND_U64 => store_ind_u64,
    LOAD_IND_U8 => load_ind_u8,
    LOAD_IND_I8 => load_ind_i8,
    LOAD_IND_U16 => load_ind_u16,
    LOAD_IND_I16 => load_ind_i16,
    LOAD_IND_U32 => load_ind_u32,
    LOAD_IND_I32 => load_ind_i32,
    LOAD_IND_U64 => load_ind_u64,
    ADD_IMM_32 => add_imm_32,
    AND_IMM => and_imm,
    XOR_IMM => xor_imm,
    OR_IMM => or_imm,
    MUL_IMM_32 => mul_imm_32,
    SET_LT_U_IMM => set_lt_u_imm,
    SET_LT_S_IMM => set_lt_s_imm,
    SHLO_L_IMM_32 => shlo_l_imm_32,
    SHLO_R_IMM_32 => shlo_r_imm_32,
    SHAR_R_IMM_32 => shar_r_imm_32,
    NEG_ADD_IMM_32 => neg_add_imm_32,
    SET_GT_U_IMM => set_gt_u_imm,
    SET_GT_S_IMM => set_gt_s_imm,
    SHLO_L_IMM_ALT_32 => shlo_l_imm_alt_32,
    SHLO_R_IMM_ALT_32 => shlo_r_imm_alt_32,
    SHAR_R_IMM_ALT_32 => shar_r_imm_alt_32,
    CMOV_IZ_IMM => cmov_iz_imm,
    CMOV_NZ_IMM => cmov_nz_imm,
    ADD_IMM_64 => add_imm_64,
    MUL_IMM_64 => mul_imm_64,
    SHLO_L_IMM_64 => shlo_l_imm_64,
    SHLO_R_IMM_64 => shlo_r_imm_64,
    SHAR_R_IMM_64 => shar_r_imm_64,
    NEG_ADD_IMM_64 => neg_add_imm_64,
    SHLO_L_IMM_ALT_64 => shlo_l_imm_alt_64,
    SHLO_R_IMM_ALT_64 => shlo_r_imm_alt_64,
    SHAR_R_IMM_ALT_64 => shar_r_imm_alt_64,
    ROT_R_64_IMM => rot_r_64_imm,
    ROT_R_64_IMM_ALT => rot_r_64_imm_alt,
    ROT_R_32_IMM => rot_r_32_imm,
    ROT_R_32_IMM_ALT => rot_r_32_imm_alt,
    BRANCH_EQ => branch_eq,
    BRANCH_NE => branch_ne,
    BRANCH_LT_U => branch_lt_u,
    BRANCH_LT_S => branch_lt_s,
    BRANCH_GE_U => branch_ge_u,
    BRANCH_GE_S => branch_ge_s,
    LOAD_IMM_JUMP_IND => load_imm_jump_ind,
    ADD_32 => add_32,
    SUB_32 => sub_32,
    MUL_32 => mul_32,
    DIV_U_32 => div_u_32,
    DIV_S_32 => div_s_32,
    REM_U_32 => rem_u_32,
    REM_S_32 => rem_s_32,
    SHLO_L_32 => shlo_l_32,
    SHLO_R_32 => shlo_r_32,
    SHAR_R_32 => shar_r_32,
    ADD_64 => add_64,
    SUB_64 => sub_64,
    MUL_64 => mul_64,
    DIV_U_64 => div_u_64,
    DIV_S_64 => div_s_64,
    REM_U_64 => rem_u_64,
    REM_S_64 => rem_s_64,
    SHLO_L_64 => shlo_l_64,
    SHLO_R_64 => shlo_r_64,
    SHAR_R_64 => shar_r_64,
    AND => and,
    XOR => xor,
    OR => or,
    MUL_UPPER_S_S => mul_upper_s_s,
    MUL_UPPER_U_U => mul_upper_u_u,
    MUL_UPPER_S_U => mul_upper_s_u,
    SET_LT_U => set_lt_u,
    SET_LT_S => set_lt_s,
    CMOV_IZ => cmov_iz,
    CMOV_NZ => cmov_nz,
    ROT_L_64 => rot_l_64,
    ROT_L_32 => rot_l_32,
    ROT_R_64 => rot_r_64,
    ROT_R_32 => rot_r_32,
    AND_INV => and_inv,
    OR_INV => or_inv,
    XNOR => xnor,
    MAX => max,
    MAX_U => max_u,
    MIN => min,
    MIN_U => min_u,
};


#[inline(always)]
fn dispatch_instr(index: u8, program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {
    FUNCTION_ARRAY[index as usize](program, pc, gas, ram, reg)
}

static INDEX: LazyLock<Mutex<u32>> = LazyLock::new(|| {Mutex::new(0)});

fn single_step_pvm(program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {

    //log::trace!("pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), program.code[pvm_ctx.pc.clone() as usize], pvm_ctx.gas, pvm_ctx.reg);

    let opcode_copy = program.code[pc.clone() as usize];

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