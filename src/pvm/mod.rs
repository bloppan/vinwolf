pub mod isa;

use std::collections::HashMap;
use std::u8;

use crate::constants::{NUM_REG, PAGE_SIZE};
use crate::types::{Context, ExitReason, PageFlags, Page, PageMap, Program, RamMemory, PageTable};
use crate::utils::codec::generic::{seq_to_number, decode_to_bits};
use crate::utils::codec::FromLeBytes;
use isa::two_reg::*;
use isa::two_reg_one_imm::*;
use isa::three_reg::*;
use isa::one_reg_one_ext_imm::*;
use isa::two_imm::*;
use isa::one_reg_one_imm::*;
use isa::one_reg_two_imm::*;


impl Default for Context {
    fn default() -> Self {
        Context {
            pc: 0,
            gas: 0,
            reg: [0; NUM_REG as usize],
            page_table: PageTable::default(),
        }
    }
}

impl Default for PageTable {
    fn default() -> Self {
        PageTable {
            pages: HashMap::new(),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            flags: PageFlags::default(),
            data: Box::new([0u8; PAGE_SIZE as usize]),
        }
    }
}

impl Default for PageFlags {
    fn default() -> Self {
        PageFlags {
            is_writable: false,
            referenced: false,
            modified: false,
        }
    }
}

impl Default for PageMap {
    fn default() -> Self {
        PageMap {
            address: 0,
            length: 0,
            is_writable: false,
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Program {
            code: vec![],
            bitmask: vec![],
            jump_table: vec![],
        }
    }
}
fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    //println!("next instruction = {}", program.code[pvm_ctx.pc as usize]);
    pvm_ctx.gas -= 1;

    let exit_reason = match program.code[pvm_ctx.pc as usize] {  // Next instruction
        //20_u8   => { load_imm_64(pvm_ctx, program) },
        30_u8   => { store_imm_u8(pvm_ctx, program) },
        31_u8   => { store_imm_u16(pvm_ctx, program) },
        32_u8   => { store_imm_u32(pvm_ctx, program) },
        /*51_u8   => { load_imm(pvm_ctx, program) },
        52_u8   => { load_u8(pvm_ctx, program) },*/
        /*53_u8   => { load_i8(pvm_ctx, program) },
        54_u8   => { load_u16(pvm_ctx, program) },
        55_u8   => { load_i16(pvm_ctx, program) },
        56_u8   => { load_u32(pvm_ctx, program) },
        57_u8   => { load_i32(pvm_ctx, program) },
        58_u8   => { load_u64(pvm_ctx, program) },*/
        59_u8   => { store_u8(pvm_ctx, program) },
        60_u8   => { store_u16(pvm_ctx, program) },
        61_u8   => { store_u32(pvm_ctx, program) },
        62_u8   => { store_u64(pvm_ctx, program) },
        70_u8   => { store_imm_ind_u8(pvm_ctx, program) },
        71_u8   => { store_imm_ind_u16(pvm_ctx, program) },
        72_u8   => { store_imm_ind_u32(pvm_ctx, program) },
        73_u8   => { store_imm_ind_u64(pvm_ctx, program) },
        /*100_u8  => { move_reg(pvm_ctx, program) }, 
        131_u8  => { add_imm_32(pvm_ctx, program) }, 
        132_u8  => { and_imm(pvm_ctx, program) },
        133_u8  => { xor_imm(pvm_ctx, program) },
        190_u8  => { add_32(pvm_ctx, program) },
        191_u8  => { sub_32(pvm_ctx, program) },
        201_u8  => { sub_64(pvm_ctx, program) },
        210_u8  => { and(pvm_ctx, program) },
        211_u8  => { xor(pvm_ctx, program) },*/

        /*8_u8    => { add(pvm_ctx, program) },
        2_u8    => { add_imm(pvm_ctx, program) },
        23_u8   => { and(pvm_ctx, program) },
        18_u8   => { and_imm(pvm_ctx, program) },
        4_u8    => { load_imm(pvm_ctx, program) },
        7_u8    => { branch_eq_imm(pvm_ctx, program) },*/
        0_u8    => { return ExitReason::trap },    // Trap
        _       => { return ExitReason::trap },   // Panic
    };

    return exit_reason;
}

pub fn invoke_pvm(
                    pvm_ctx: &mut Context, 
                    program_blob: &[u8], 
                    ) 
    -> ExitReason 
{

    let _j_size = program_blob[0];          // Dynamic jump table size
    let j: Vec<u8> = vec![];                    // Dynamic jump table
    let _z = program_blob[1];               // Jump octects length
    let program_size = program_blob[2] as u32;
    println!("Program size = {}", program_size);
    let program = Program {
        code: program_blob[3..3 + program_size as usize].to_vec() // Instruction vector
            .into_iter()
            .chain(std::iter::repeat(0).take(25)) // Sequence of zeroes suffixed to ensure that no out-of-bounds access is possible
            .collect(),
        bitmask: {
            /*println!("resto = {}", 8 - (program_size as usize % 8));
            println!("Data len = {}", 25 - (program_size as usize % 8));*/
            decode_to_bits(&program_blob[3 + program_size as usize..program_blob.len()].to_vec())
                .into_iter()
                .enumerate()
                .map(|(i, bit)| if i >= program_size as usize { true } else { bit })
                .chain(std::iter::repeat(true).take(25 - (8 - program_size as usize % 8)))
                .collect()
        },
        jump_table: j, // Dynamic jump table
    };
    
    println!("Program sequence = {:?}", program.code);
    println!("Bitmask sequence = {:?}", program.bitmask);
    println!("Program len = {} Bitmask len = {}", program.code.len(), program.bitmask.len());
    println!("Program sequence size = {} | bitmask sequence size = {}", program.code.len(), program.bitmask.len());
    //let mut pvm_ctx = Context { pc, gas, reg, ram }; // PVM context
    let mut exit_reason;

    while pvm_ctx.gas > 0 {

        exit_reason = single_step_pvm(pvm_ctx, &program);
        // TODO revisar esto
        match exit_reason {
            ExitReason::Continue => {
                pvm_ctx.pc += 1;
            },
            ExitReason::PageFault(_) => {
                pvm_ctx.gas -= 1; 
                return ExitReason::trap;
            },
            ExitReason::trap => {
                return exit_reason;
            },
            _ => { return exit_reason; },
        }
    }

    return ExitReason::OutOfGas;
}

