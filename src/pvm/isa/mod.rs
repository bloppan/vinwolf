use std::collections::HashMap;

use crate::types::{
    Context, MemoryChunk, PageFlags, RamAccess, RamAddress, RamMemory, Page, PageTable, RegSize, Program, ExitReason
};
use crate::constants::{NUM_PAGES, PAGE_SIZE};
use crate::utils::codec::EncodeSize;
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

impl Default for RamMemory {
    fn default() -> Self {
        let mut v: Vec<Option<Page>> = Vec::with_capacity(NUM_PAGES as usize);
        for _ in 0..NUM_PAGES {
            v.push(None);
        }
        RamMemory {
            pages: v.into_boxed_slice(),
        }
    }
}

impl RamMemory {
    pub fn insert(&mut self, address: RamAddress, value: u8) {
        let page_target = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;
        println!("Inserting value {} at address {}", value, address);
        if let Some(page) = self.pages[page_target as usize].as_mut() {
            println!("Inserting value {} at address {}", value, address);
            page.data[offset as usize] = value;
        }
    }
}


impl Default for MemoryChunk {
    fn default() -> Self {
        MemoryChunk {
            address: 0,
            contents: vec![],
        }
    }
}


fn skip(i: &u64, k: &[bool]) -> u64 {
    let mut j = i + 1;
    //println!("k = {:?}", k);
    while j < k.len() as u64 && k[j as usize] == false {
        j += 1;
    }
    //println!("j = {}", j-1);
    std::cmp::min(24, j - 1)
}


pub fn extend_sign(v: &[u8], n: usize) -> RegSize {

    if ![1, 2, 3, 4, 8].contains(&n) {
        return 0;
    }

    let mut buffer = [0u8; 8];
    buffer[..n].copy_from_slice(&v[..n]);

    let x = u64::from_le_bytes(buffer);
    let sign_bit = (x >> ((8 * n) - 1)) & 1;

    if sign_bit == 1 {
        x | (!0u64 << (8 * n - 1))
    } else {
        x
    }
}

pub fn unsigned_to_signed(input: &[u8], n: usize) -> i64 {

    let mut buffer = [0u8; 8];
    buffer[..n].copy_from_slice(&input[..n]);
    let a = u64::from_le_bytes(buffer);

    if a < (1 << (8 * n - 1)) {
        a as i64
    } else {
        (a as i128 - (1i128 << (8 * n))) as i64
    }
}

pub fn signed_to_unsigned(input: i64, n: usize) -> Vec<u8> {

    let a = (((1u128 << (8 * n)).wrapping_add(input as u128)) % (1 << (8 * n) as u128)) as u64;
    a.to_le_bytes().to_vec()
}

pub fn _store<T>(pvm_ctx: &mut Context, program: &Program, address: RamAddress, value: RegSize) -> ExitReason {
    if let Err(address) = check_page_fault::<T>(pvm_ctx, address as RamAddress, RamAccess::Write) {
        println!("Page fault at address {}", address);
        return ExitReason::PageFault(address);
    }
    
    for (i, byte) in value.encode_size(std::mem::size_of::<T>()).iter().enumerate() {
        let page_address = address.wrapping_add(i as RamAddress) / PAGE_SIZE;
        let offset = address.wrapping_add(i as RamAddress) % PAGE_SIZE;
        if let Some(page) = pvm_ctx.page_table.pages.get_mut(&page_address) {
            page.data[offset as usize] = *byte;
            page.flags.modified = true;
        }
    }
    pvm_ctx.pc = skip(&pvm_ctx.pc, &program.bitmask);
    ExitReason::Continue
}


pub fn check_page_fault<T>(pvm_ctx: &Context, address: RamAddress, access: RamAccess) -> Result<(), RamAddress> {

    println!("Checking page fault at address {}", address);
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

fn basic_block_seq(pc: &u64, k: &[bool]) -> u64 {
    return 1 + skip(pc, k) as u64;
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
            (vec![0x00, 0x00, 0x02, 0x18], 3, 0x020000u64),
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
            let result = extend_sign(&input, size);
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
            let result = unsigned_to_signed(&input, n);
            assert_eq!(expected, result, "Fallo en el caso con input: {:?}, n: {}", input, n);
        }
    }

    #[test]
    fn test_signed_to_unsigned() {
        let test_cases = vec![
            (0, 1, vec![0x00]),
            (-1, 1, vec![0xFF]),
            (127, 1, vec![0x7F]),
            (-128, 1, vec![0x80]),

            (0, 2, vec![0x00, 0x00]),
            (-1, 2, vec![0xFF, 0xFF]),
            (32767, 2, vec![0xFF, 0x7F]),
            (-32768, 2, vec![0x00, 0x80]),

            (0, 4, vec![0x00, 0x00, 0x00, 0x00]),
            (-1, 4, vec![0xFF, 0xFF, 0xFF, 0xFF]),
            (2147483647, 4, vec![0xFF, 0xFF, 0xFF, 0x7F]),
            (-2147483648, 4, vec![0x00, 0x00, 0x00, 0x80]),

            (0, 8, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (-1, 8, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
            (9223372036854775807, 8, vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]),
            (-9223372036854775808, 8, vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
        ];

        for (input, n, expected) in test_cases {
            let result = signed_to_unsigned(input, n);
            assert_eq!(result[..n], expected[..], "Fallo en el caso con input: {}, n: {}", input, n);
        }
    }
}