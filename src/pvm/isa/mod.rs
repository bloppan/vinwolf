use std::collections::HashMap;

use crate::types::{
    Context, RamAccess, RamAddress, RegSize, RegSigned, Program, ExitReason
};
use crate::constants::{NUM_PAGES, PAGE_SIZE};
use crate::utils::codec::{EncodeSize, FromLeBytes};
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

pub fn skip(i: &u64, k: &[bool]) -> u64 {
    let mut j = *i + 1;
    //println!("k = {:?}", k);
    while j < k.len() as u64 && k[j as usize] == false {
        j += 1;
        //println!("j = {}", j);
    }
    //println!("j = {}", j-1);
    std::cmp::min(24, (j - 1).saturating_sub(*i))
}


pub fn extend_sign(le_bytes: &[u8]) -> RegSize {

    let num_bytes = le_bytes.len();

    if ![1, 2, 3, 4, 8].contains(&num_bytes) {
        return 0;
    }

    let x = decode::<u64>(le_bytes);
    let sign_bit = (x >> ((8 * num_bytes) - 1)) & 1;

    if sign_bit == 1 {
        return x | (!0u64 << (8 * num_bytes - 1));
    } 

    return x;
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

fn unsigned_to_signed(le_bytes: &[u8]) -> i64 {
    let nbits = 8 * le_bytes.len();
    let a = decode::<u64>(le_bytes); 
    let shift = 64 - nbits;
    ((a << shift) as i64) >> shift
}


fn signed_to_unsigned(le_bytes: &[u8]) -> Vec<u8> {
    let n = le_bytes.len();
    let sign_bit_set = (le_bytes[n - 1] & 0x80) != 0;
    let mut extended = [0u8; 8];
    extended[..n].copy_from_slice(le_bytes);

    if sign_bit_set {
        for b in &mut extended[n..] {
            *b = 0xFF;
        }
    }

    let signed = i64::from_le_bytes(extended);
    let unsigned = signed as u64;

    return unsigned.to_le_bytes()[..n].to_vec();
}

pub fn _load<T>(pvm_ctx: &mut Context, address: RamAddress, reg: RegSize, signed: bool) -> ExitReason {

    if let Err(address) = check_page_fault::<T>(pvm_ctx, address as RamAddress, RamAccess::Read) {
        return ExitReason::PageFault(address);
    }
    let mut value: Vec<u8> = Vec::new();

    for i in 0..std::mem::size_of::<T>() {
        let page_target = address.wrapping_add(i as RamAddress) / PAGE_SIZE; 
        let offset = address.wrapping_add(i as RamAddress) % PAGE_SIZE;
        if let Some(page) = pvm_ctx.page_table.pages.get_mut(&page_target) {
            value.push(page.data[offset as usize] as u8); 
            page.flags.referenced = true;
        } else {
            return ExitReason::PageFault(address);
        }
    }
    if signed {
        pvm_ctx.reg[reg as usize] = extend_sign(&value);
    } else {
        pvm_ctx.reg[reg as usize] = decode::<RegSize>(&value);
    }
    ExitReason::Continue
}


pub fn _store<T>(pvm_ctx: &mut Context, program: &Program, address: RamAddress, value: RegSize) -> ExitReason {

    if let Err(address) = check_page_fault::<T>(pvm_ctx, address as RamAddress, RamAccess::Write) {
        return ExitReason::PageFault(address);
    }
    
    for (i, byte) in value.encode_size(std::mem::size_of::<T>()).iter().enumerate() {
        let page_address = address.wrapping_add(i as RamAddress) / PAGE_SIZE;
        let offset = address.wrapping_add(i as RamAddress) % PAGE_SIZE;
        if let Some(page) = pvm_ctx.page_table.pages.get_mut(&page_address) {
            page.data[offset as usize] = *byte;
            page.flags.modified = true;
        } else {
            return ExitReason::PageFault(address.wrapping_add(i as RamAddress));
        }
    }
    ExitReason::Continue
}


pub fn check_page_fault<T>(pvm_ctx: &Context, address: RamAddress, access: RamAccess) -> Result<(), RamAddress> {

    for i in 0..std::mem::size_of::<T>() {
        let page_target = address.wrapping_add(i as RamAddress) / PAGE_SIZE;

        if let Some(page) = pvm_ctx.page_table.pages.get(&page_target) {
            if access == RamAccess::Write && !page.flags.is_writable {
                return Err(address);
            }
        } else {
            return Err(address);
        }
    }

    return Ok(());
}

pub fn djump(a: &RegSize, pc: &mut RegSize, program: &Program) -> ExitReason {
    if *a == 0xFFFF0000 {
        println!("Halt");
        println!("pc = {}", pc);
        return ExitReason::halt;
    } else if *a == 0 ||  *a as usize > program.bitmask.len() * 2 || a % 2 != 0 {
        println!("Trap");
        println!("pc = {}", pc);
        ExitReason::trap
    } else {
        let jump = (*a as usize / 2) - 1;
        println!("Jumping to jump table pos {}", jump);
        *pc = program.jump_table[jump] as u64 - 1;
        println!("pc = {}", pc);
        ExitReason::Continue
    }    
}

fn basic_block_seq(pc: &RegSize, k: &[bool]) -> RegSize {
    return 1 + skip(pc, k) as RegSize;
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
        ];
    
        for (input, size, expected) in test_cases {
            let result = extend_sign(&input);
            assert_eq!(expected, result, "Fail input {:?} size {}", input, size);
        }
    }

    #[test]
    fn test_unsigned_to_signed() {
        let test_cases = vec![
            (vec![0x00], 1, 0i64),  
            (vec![0x7F], 1, 127i64), 
            (vec![0x80], 1, -128i64),
            (vec![0xFF], 1, -1i64),  

            (vec![0x00, 0x00], 2, 0i64), 
            (vec![0xFF, 0x7F], 2, 32767i64), 
            (vec![0x00, 0x80], 2, -32768i64),
            (vec![0xFF, 0xFF], 2, -1i64), 

            (vec![0x00, 0x00, 0x00, 0x00], 4, 0i64), 
            (vec![0xFF, 0xFF, 0xFF, 0x7F], 4, 2147483647i64), 
            (vec![0x00, 0x00, 0x00, 0x80], 4, -2147483648i64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF], 4, -1i64), 

            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 8, 0i64), 
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F], 8, 9223372036854775807i64), 
            (vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80], 8, -9223372036854775808i64),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 8, -1i64), 
        ];

        for (input, n, expected) in test_cases {
            let result = unsigned_to_signed(&input);
            assert_eq!(expected, result, "Fallo en el caso con input: {:?}, n: {}", input, n);
        }
    }

    #[test]
    fn test_signed_to_unsigned() {
        let test_cases = vec![
            (0i64, 1, vec![0x00]),
            (-1i64, 1, vec![0xFF]),
            (127i64, 1, vec![0x7F]),
            (-128i64, 1, vec![0x80]),

            (0i64, 2, vec![0x00, 0x00]),
            (-1i64, 2, vec![0xFF, 0xFF]),
            (32767i64, 2, vec![0xFF, 0x7F]),
            (-32768i64, 2, vec![0x00, 0x80]),

            (0i64, 4, vec![0x00, 0x00, 0x00, 0x00]),
            (-1i64, 4, vec![0xFF, 0xFF, 0xFF, 0xFF]),
            (2147483647i64, 4, vec![0xFF, 0xFF, 0xFF, 0x7F]),
            (-2147483648i64, 4, vec![0x00, 0x00, 0x00, 0x80]),

            (0i64, 8, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (-1i64, 8, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            (9223372036854775807i64, 8, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]),
            (-9223372036854775808i64, 8, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ];

        for (input, n, expected) in test_cases {
            let result = signed_to_unsigned(&input.to_le_bytes());
            assert_eq!(result[..n], expected[..], "Failed on input: {}, n: {}", input, n);
        }
    }

    #[test]
    fn test_signed() {
        let test_cases = vec![
            (1, 1, 1i64),
            (255, 1, -1i64),
            (127, 1, 127i64),
            (128, 1, -128i64),
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
            (i64::MAX, 8, 0x7FFFFFFFFFFFFFFF_u64), 
        ];
        
        for (input, n, expected) in test_cases {
            let result = unsigned(input, n);
            assert_eq!(result, expected, "Failed on input: {}, n: {}", input, n);
        }
    }
    
    
}