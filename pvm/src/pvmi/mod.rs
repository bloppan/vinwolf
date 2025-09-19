use crate::pvm_types::{Gas, RegSize, RamMemory, Registers, RamAddress};
use crate::pvm_types::{ExitReason, Program};
use constants::pvm::{*};
use codec::EncodeSize;
use codec::generic_codec::decode;
use utils::log;
use std::sync::{Mutex, LazyLock};

pub mod no_arg;
pub mod one_imm;
pub mod one_offset;
pub mod one_reg_one_imm;
pub mod one_reg_one_imm_one_offset;
pub mod one_reg_two_imm;
pub mod three_reg;
pub mod two_imm;
pub mod two_reg_one_imm;
pub mod two_reg_one_offset;
pub mod two_reg_two_imm;
pub mod two_reg;
pub mod one_reg_one_ext_imm;

use one_offset::*;
use no_arg::*;
use one_imm::*;
use two_reg::*;
use two_reg_one_imm::*;
use two_reg_two_imm::*;
use two_reg_one_offset::*;
use three_reg::*;
use one_reg_one_ext_imm::*;
use two_imm::*;
use one_reg_one_imm::*;
use one_reg_two_imm::*;
use one_reg_one_imm_one_offset::*;

pub const PAGE_SHIFT: RamAddress = PAGE_SIZE.trailing_zeros();
pub const PAGE_MASK: RamAddress = PAGE_SIZE - 1;
pub const BOUNDS_MASK: RamAddress = NUM_PAGES - 1;

#[macro_export] macro_rules! page_index {
    ($addr:expr) => {
        $addr >> crate::pvmi::PAGE_SHIFT
    };
}

#[macro_export] macro_rules! page_offset {
    ($addr:expr) => {
        $addr & crate::pvmi::PAGE_MASK
    };
}

#[macro_export] macro_rules! mem_bounds {
    ($addr:expr) => {
        $addr & crate::pvmi::BOUNDS_MASK
    };
}

pub fn invoke_pvm(program: &Program, pc: &mut RegSize, gas: &mut Gas, ram: &mut RamMemory, reg: &mut Registers) -> ExitReason {

    log::debug!("Invoke inner pvm");

    loop {
        
        *gas -= 1;

        if *gas < 0 {
            return ExitReason::OutOfGas;
        }
        
        let exit_reason = dispatch_instr(program.code[*pc as usize], program, pc, gas, ram, reg);
        //let exit_reason = single_step_pvm(&program, pc, gas, ram, reg);

        match exit_reason {
            ExitReason::Continue => {
                log::trace!("Exit: {:?} pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", exit_reason, *pc, program.code[*pc as usize], gas, reg);
                continue;
            },
            ExitReason::OutOfGas => {
                log::debug!("Exit: {:?} pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", exit_reason, *pc, program.code[*pc as usize], gas, reg);
                return ExitReason::OutOfGas;
            },
            ExitReason::Halt => {
                log::debug!("Exit: {:?} pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", exit_reason, *pc, program.code[*pc as usize], gas, reg);
                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien) TODO
                return ExitReason::Halt;
            },
            _ => { 
                log::debug!("Exit: {:?} pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", exit_reason, *pc, program.code[*pc as usize], gas, reg);
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

#[allow(dead_code)]
static INDEX: LazyLock<Mutex<u32>> = LazyLock::new(|| {Mutex::new(0)});


static BASIC_BLOCK_TERMINATORS: &[u8] = &[
    TRAP,
    FALLTHROUGH,
    JUMP,
    JUMP_IND,
    LOAD_IMM_JUMP,
    LOAD_IMM_JUMP_IND,
    BRANCH_EQ,
    BRANCH_NE,
    BRANCH_GE_U,
    BRANCH_GE_S,
    BRANCH_LT_U,
    BRANCH_LT_S,
    BRANCH_EQ_IMM,
    BRANCH_NE_IMM,
    BRANCH_LT_U_IMM,
    BRANCH_LT_S_IMM,
    BRANCH_LE_U_IMM,
    BRANCH_LE_S_IMM,
    BRANCH_GE_U_IMM,
    BRANCH_GE_S_IMM,
    BRANCH_GT_U_IMM,
    BRANCH_GT_S_IMM,
];

#[inline(always)]
pub fn begin_basic_block(program: &Program, pc: &RegSize, next_instr: usize) -> bool {
    
    if *pc == 0 {
        return true;
    }

    let byte_index = next_instr >> 3;
    let bit_offset = (next_instr & 0b111) as u8;

    if byte_index >= program.bitmask.len() {
        return false; 
    }

    let mask = 1 << bit_offset; 
    let is_bit_set = (program.bitmask[byte_index] & mask) != 0;

    if is_bit_set && BASIC_BLOCK_TERMINATORS.contains(&program.code[*pc as usize]) {
        return true;
    }

    return false;
}

#[inline(always)]
pub fn skip(i: &u64, k: &[u8]) -> u64 {

    let byte_index = *i >> 3;
    let bit_offset = (*i & 0b111) as u8;

    let mut bitmask_array = [0u8; 4];
    bitmask_array.copy_from_slice(&k[byte_index as usize..byte_index as usize + 4]);
    let bitmask: u32 = u32::from_le_bytes(bitmask_array) >> (bit_offset + 1);
    bitmask.trailing_zeros() as u64
    //std::cmp::min(24, bitmask.trailing_zeros() as u64)
}

pub fn extend_sign(le_bytes: &[u8], n: usize) -> RegSize {
    match n {
        1 => (le_bytes[0] as i8 as i64) as RegSize,
        2 => {
            let v = u16::from_le_bytes([le_bytes[0], le_bytes[1]]);
            (v as i16 as i64) as RegSize
        }
        3 => {
            let v = (le_bytes[0] as u32)
                | ((le_bytes[1] as u32) << 8)
                | ((le_bytes[2] as u32) << 16);
            (((v << 8) as i32 >> 8) as i64) as RegSize
        }
        4 => {
            let v = u32::from_le_bytes([le_bytes[0], le_bytes[1], le_bytes[2], le_bytes[3]]);
            (v as i32 as i64) as RegSize
        }
        8 => {
            let v = u64::from_le_bytes([
                le_bytes[0], le_bytes[1], le_bytes[2], le_bytes[3],
                le_bytes[4], le_bytes[5], le_bytes[6], le_bytes[7],
            ]);
            (v as i64) as RegSize
        }
        _ => 0,
    }
}

pub fn signed(a: u64, n: usize) -> i64 {
    let bits = match n { 
        1..=8 => (n * 8) as u32, 
        _ => return 0 
    };
    let masked = if bits == 64 { a } else { a & ((1u64 << bits) - 1) };
    ((masked << (64 - bits)) as i64) >> (64 - bits)
}

pub fn unsigned(a: i64, n: usize) -> u64 {
    match n {
        1..=8 => {
            let bits = (n * 8) as u32;
            let ua = a as u64;
            if bits == 64 { ua } else { ua & ((1u64 << bits) - 1) }
        }
        _ => 0,
    }
}

pub fn _branch(
    pc: &mut RegSize, 
    program: &Program, 
    n: i64,
) -> ExitReason {

    // Check if the jump is out of bounds
    if n < 0 || n as usize >= program.code.len() {
        log::error!("Panic: jump out of bounds");
        return ExitReason::Panic;
    }

    // Check for the beginning of a basic-block
    if !begin_basic_block(program, pc, *pc as usize + 1 + skip(pc, &program.bitmask) as usize) {
        log::error!("Panic: not a basic block");
        return ExitReason::Panic;
    }

    //pvm_ctx.pc = (n - 1) as RegSize;
    *pc = n as RegSize;
        
    return ExitReason::Continue;
}

pub fn _load<T>(
    program: &Program, 
    pc: &mut RegSize, 
    ram: &mut RamMemory, 
    reg: &mut Registers, 
    address: RamAddress, 
    reg_target: RegSize, 
    signed: bool
) -> ExitReason {

    let num_bytes = std::mem::size_of::<T>() as RamAddress;

    if let Err(exit_reason) = ram.is_readable(address, num_bytes) {
        return exit_reason;
    }

    let n = std::mem::size_of::<T>();
    let value = ram.read(address, num_bytes as RamAddress);

    if signed {
        reg[reg_target as usize] = extend_sign(&value, n);
        *pc += skip(pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    reg[reg_target as usize] = decode::<RegSize>(&value, n);
    *pc += skip(pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}

pub fn _store<T>(
    program: &Program, 
    pc: &mut RegSize,
    ram: &mut RamMemory,
    address: RamAddress, 
    value: RegSize
) -> ExitReason {

    let num_bytes = std::mem::size_of::<T>();

    if let Err(exit_reason) = ram.is_writable(address, num_bytes as RamAddress) {
        return exit_reason;
    }
    
    ram.write(address, &value.encode_size(num_bytes));
    *pc += skip(pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn djump(a: &RegSize, pc: &mut RegSize, program: &Program) -> ExitReason {

    let jump_table_position = (*a as usize / JUMP_ALIGNMENT).saturating_sub(1);

    if *a == 0xFFFF0000 {
        log::info!("JUMP TO HALT");
        return ExitReason::Halt;
    } else if *a == 0 
            || *a as usize > program.jump_table.len() * JUMP_ALIGNMENT 
            || *a as usize % JUMP_ALIGNMENT != 0 
            || !begin_basic_block(program, pc, program.jump_table[jump_table_position]) 
    {
        ExitReason::Panic
    } else {        
        *pc = program.jump_table[jump_table_position] as u64;
        ExitReason::Continue
    }    
}

#[allow(dead_code)]
fn smod(a: i64, b: i64) -> i64 {
    
    if b == 0 {
        return a;
    }

    let result = a.abs() % b.abs();

    if a >= 0 {
        return result;
    } 

    return -result;
}

#[cfg(test)]
mod test { 
    use super::*;

    #[test]
    fn test_extend_sign() {
        let test_cases = vec![
            (vec![0x01], 1, 1u64),
            (vec![0xFF], 1, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x40], 1, 0x40),
            (vec![0x80], 1, 0xFFFFFFFFFFFFFF80),
            (vec![0x01, 0x00], 2, 1u64),
            (vec![0x80, 0xFF], 2, 0xFFFFFFFFFFFFFF80),
            (vec![0xFF, 0xFF], 2, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x80], 2, 0xFFFFFFFFFFFF8000u64),
            (vec![0x00, 0x00, 0x02], 3, 0x020000u64),
            (vec![0x01, 0x00, 0x00], 3, 1u64),
            (vec![0xFF, 0xFF, 0xFF], 3, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x80], 3, 0xFFFFFFFFFF800000u64),
            (vec![0xD4, 0xFE, 0xFF], 3, 0xFFFFFFFFFFFFFED4u64),
            (vec![0x01, 0x00, 0x00, 0x00], 4, 1u64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF], 4, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x00, 0x80], 4, 0xFFFFFFFF80000000u64),
            (vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 8, 1u64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 8, 0xFFFFFFFFFFFFFFFFu64),
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 8, 0x8000000000000000u64),
            (vec![0x80, 0xC1, 0x1D, 0xF3, 0x2A, 0x00, 0x00, 0x00], 8, 184467440000),
        ];
    
        for (input, size, expected) in test_cases {
            let result = extend_sign(&input, size);
            assert_eq!(expected, result, "Fail input {:?} size {}", input, size);
        }
    }

    #[test]
    fn test_signed() {
        let test_cases = vec![
            (1, 1, 1i64),
            (255, 1, -1i64),
            (127, 1, 127i64),
            (128, 1, -128i64),
            (129, 1, -127i64),
            (130, 1, -126i64),
            (254, 1, -2i64),
            (65535, 2, -1i64),
            (32768, 2, -32768i64),
            (32767, 2, 32767i64),
            (2147483648, 4, -2147483648i64),
            (2147483647, 4, 2147483647i64),
            (0, 8, 0i64),
            (0xFFFFFFFFFFFFFFFF, 8, -1i64),
            (1, 8, 1i64),
            (0x8000000000000000, 8, -9223372036854775808i64),
            (9223372036854775807, 8, 9223372036854775807i64),
            (0x8000000000000001, 8, -9223372036854775807i64),
            (0x8000000000000000, 8, -9223372036854775808i64),
            (0x7FFFFFFFFFFFFFFF, 8, 9223372036854775807i64),
        ];
        for (input, n, expected) in test_cases {
            let result = signed(input, n);
            assert_eq!(result, expected, "Failed on input: {}, n: {}", input, n);
        }
    }

    #[test]
    fn test_unsigned() {
        let test_cases: Vec<(i64, usize, u64)> = vec![
            (0, 1, 0u64),
            (-1, 1, 255u64),       
            (127, 1, 127u64),      
            (-128, 1, 128u64),     
            (255, 1, 255u64),      
            (-1, 2, 65535u64),     
            (-32768, 2, 32768u64), 
            (32767, 2, 32767u64),  
            (-2147483648, 4, 2147483648u64), 
            (2147483647, 4, 2147483647u64),  
            (0, 8, 0u64),
            (-1, 8, 0xFFFFFFFFFFFFFFFFu64),
            (1, 8, 1u64),
            (-9223372036854775808, 8, 0x8000000000000000u64),
            (9223372036854775807, 8, 9223372036854775807u64),
            (-9223372036854775807, 8, 0x8000000000000001u64),
            (i64::MIN, 8, 0x8000000000000000u64),  
            (i64::MAX, 8, 0x7FFFFFFFFFFFFFFFu64), 
        ];
        
        for (input, n, expected) in test_cases {
            let result = unsigned(input, n);
            assert_eq!(result, expected, "Failed on input: {}, n: {}", input, n);
        }
    }

    #[test]
    fn test_smod() {
        assert_eq!(smod(10, 3), 1);
        assert_eq!(smod(-10, 3), -1);
        assert_eq!(smod(10, -3), 1);
        assert_eq!(smod(-10, -3), -1);

        assert_eq!(smod(5, 2), 1);
        assert_eq!(smod(-5, 2), -1);
        assert_eq!(smod(5, -2), 1);
        assert_eq!(smod(-5, -2), -1);

        assert_eq!(smod(0, 3), 0);
        assert_eq!(smod(0, -3), 0);
        assert_eq!(smod(42, 0), 42);
        assert_eq!(smod(-42, 0), -42);

        assert_eq!(smod(i64::MAX, 3), 1);
        assert_eq!(smod(i64::MIN + 1, 2), -1);
    }
    
    /*#[test]
    fn test_address() {
        let address = (1 << 16) as RamAddress;
        let page1 = (address - 1) / PAGE_SIZE;
        let page2 = address / PAGE_SIZE;

        println!("page1 = {}, page2 = {}", page1, page2);
    }*/
    
}

#[allow(dead_code)]
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
        _                       => { log::error!("Unknown instruction!"); return ExitReason::Panic },
    };

    //log::trace!("pc = {:?}, opcode = {:03?}, index = {:05?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, INDEX.lock().unwrap(), 20_000_000_i64.saturating_sub(pvm_ctx.gas), pvm_ctx.reg);
    //*INDEX.lock().unwrap() += 1;

    return exit_reason;
}
