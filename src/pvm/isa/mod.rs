use crate::pvm;
use crate::types::{Context, RamAccess, RamAddress, RegSize, Program, ExitReason};
use crate::constants::{LOWEST_ACCESIBLE_PAGE, JUMP_ALIGNMENT, PAGE_SIZE};
use crate::utils::codec::EncodeSize;
use crate::utils::codec::generic::decode;

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

use super::{*};

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

pub fn begin_basic_block(program: &Program, pc: &RegSize, next_instr: usize) -> bool {
    
    if *pc == 0 {
        return true;
    }

    if program.bitmask[next_instr] && BASIC_BLOCK_TERMINATORS.contains(&program.code[*pc as usize]) {
        return true;
    }

    return false;
}

pub fn skip(i: &u64, k: &[bool]) -> u64 {
    let mut j = *i + 1;
    //println!("k = {:?}", k);
    while k[j as usize] == false && j < k.len() as u64 {
        j += 1;
        //println!("j = {}", j);
    }
    //println!("j = {}", j-1);
    std::cmp::min(24, (j - 1).saturating_sub(*i))
}

// TODO creo que no deberia devolver 0 
pub fn extend_sign(le_bytes: &[u8], n: usize) -> RegSize {

    if ![1, 2, 3, 4, 8].contains(&n) {
        return 0;
    }

    let x = decode::<RegSize>(le_bytes, n);

    let shift = 64 - 8 * n;
    let extended = ((x << shift) as i64 >> shift) as RegSize;

    return extended;
}

// TODO
pub fn extend_sign2<T>(value: T) -> RegSize
where 
    T: Copy + Into<i128> + From<i128> + From<RegSize>,
{
    let width = std::mem::size_of::<T>() as i32 * 8;
    let shift = 128 - width;
    let value_128: i128 = value.into();
    let extended = (value_128 << shift) >> shift;

    return extended as RegSize;
}

pub fn signed(a: u64, n: usize) -> i64 {

    if a < (1 << (8 * n - 1)) {
        return a as i64;
    }

    return (a as u128).wrapping_sub(1u128 << (8 * n)) as i64;
}

pub fn unsigned(a: i64, n: usize) -> u64 {
    let modulus = 1u128 << (8 * n);
    ((modulus.wrapping_add(a as u128)) % modulus) as u64
}

pub fn _branch(
    pvm_ctx: &mut Context, 
    program: &Program, 
    n: i64,
) -> ExitReason {

    // Check if the jump is out of bounds
    if n < 0 || n as usize >= program.code.len() {
        println!("Panic: jump out of bounds");
        return ExitReason::panic;
    }

    // Check for the beginning of a basic-block
    if !begin_basic_block(program, &pvm_ctx.pc, pvm_ctx.pc as usize + 1 + skip(&pvm_ctx.pc, &program.bitmask) as usize) {
        println!("Panic: not a basic block");
        return ExitReason::panic;
    }

    //pvm_ctx.pc = (n - 1) as RegSize;
    pvm_ctx.pc = n as RegSize;
        
    return ExitReason::Continue;
}

pub fn _load<T>(pvm_ctx: &mut Context, program: &Program, address: RamAddress, reg: RegSize, signed: bool) -> ExitReason {

    if let Err(check_error) = check_memory_access::<T>(pvm_ctx, address as RamAddress, RamAccess::Read) {
        return check_error;
    }
    let mut value: Vec<u8> = Vec::new();
    let n = std::mem::size_of::<T>();

    for i in 0..std::mem::size_of::<T>() {
        let page_target = address.wrapping_add(i as RamAddress) / PAGE_SIZE; 
        let offset = address.wrapping_add(i as RamAddress) % PAGE_SIZE;
        let byte = pvm_ctx.ram.pages[page_target as usize].as_ref().unwrap().data[offset as usize] as u8;
        value.push(byte); 
        pvm_ctx.ram.pages[page_target as usize].as_mut().unwrap().flags.referenced = true;
    }
    
    if signed {
        pvm_ctx.reg[reg as usize] = extend_sign(&value, n);
        pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
        return ExitReason::Continue;
    } 

    pvm_ctx.reg[reg as usize] = decode::<RegSize>(&value, n);
    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    return ExitReason::Continue;
}


pub fn _store<T>(pvm_ctx: &mut Context, program: &Program, address: RamAddress, value: RegSize) -> ExitReason {

    if let Err(check_error) = check_memory_access::<T>(pvm_ctx, address as RamAddress, RamAccess::Write) {
        return check_error;
    }
    
    for (i, byte) in value.encode_size(std::mem::size_of::<T>()).iter().enumerate() {
        let page_target = address.wrapping_add(i as RamAddress) / PAGE_SIZE;
        let offset = address.wrapping_add(i as RamAddress) % PAGE_SIZE;
        pvm_ctx.ram.pages[page_target as usize].as_mut().unwrap().data[offset as usize] = *byte;
        pvm_ctx.ram.pages[page_target as usize].as_mut().unwrap().flags.modified = true;
    }

    pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
    ExitReason::Continue
}

pub fn check_memory_access<T>(pvm_ctx: &mut Context, address: RamAddress, access: RamAccess) -> Result<(), ExitReason> {
    
    if pvm_ctx.pc == 34702 {
        //println!("aqui bernar address: {address} pc = {:?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), pvm_ctx.gas, pvm_ctx.reg);
    }

    /*if pvm_ctx.pc == 23352 || pvm_ctx.pc == 23355 {
        println!("aqui address: {address}");
        println!("pc = {:?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), pvm_ctx.gas, pvm_ctx.reg); 
    }*/

    //if (address / PAGE_SIZE == 1044447 && access == RamAccess::Write) || pvm_ctx.pc == 34702 {
    
    /*if pvm_ctx.gas > 9999959 || address / PAGE_SIZE == 50 && (pvm_ctx.pc == 34702  || access == RamAccess::Write) {
        println!("pc = {:?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), pvm_ctx.gas, pvm_ctx.reg); 
        println!();
        //for i in 0..(4096 / 32) {
        for i in 58..66 {
            print!("{:08}:\t", (i * 32) as u64 + (4096 as u64 * 50 as u64) as u64);
            for j in 0..32 {
                let index = i * 32 + j;
                print!("{:02x?} ", pvm_ctx.ram.pages[50].as_ref().unwrap().data[index as usize]);
            }
            println!();
        }
        println!();
    }*/

    for i in 0..std::mem::size_of::<T>() {
        //println!("address = {:?}", address);
        let page_target = address.wrapping_add(i as RamAddress) / PAGE_SIZE;
        // Check if the page is in the range of the highest inaccessible page (0xFFFF0000)
        if page_target < LOWEST_ACCESIBLE_PAGE {
            println!("Panic: page target out of bounds");
            return Err(ExitReason::panic);
        }
        // Check if the page is in the page table
        if let Some(page) = pvm_ctx.ram.pages[page_target as usize].as_ref() {
            // Check the access flags
            if page.flags.access.get(&access).is_none() {
                println!("Panic: page {page_target} access violation");
                return Err(ExitReason::panic);
            }
        } else {
            pvm_ctx.page_fault = Some(address.wrapping_add(i as RamAddress));
            return Err(ExitReason::page_fault);
            // TODO cambiar esto
            //return Err(ExitReason::PageFault(page_target));
        }
    }

    return Ok(());
}

pub fn djump(a: &RegSize, pc: &mut RegSize, program: &Program) -> ExitReason {

    let jump_table_position = (*a as usize / JUMP_ALIGNMENT).saturating_sub(1);

    if *a == 0xFFFF0000 {
        return ExitReason::Halt;
    } else if *a == 0 
            || *a as usize > program.jump_table.len() * JUMP_ALIGNMENT 
            || *a as usize % JUMP_ALIGNMENT != 0 
            || !begin_basic_block(program, pc, program.jump_table[jump_table_position]) 
    {
        ExitReason::panic
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