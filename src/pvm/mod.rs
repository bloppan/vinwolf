pub mod isa;
pub mod mm;

use std::collections::HashMap;
use std::u8;

use crate::constants::{NUM_REG, PAGE_SIZE};
use crate::types::{Context, ExitReason, PageFlags, Page, PageMap, Program, RamMemory, PageTable};
use crate::utils::codec::generic::{decode_to_bits, decode_unsigned, seq_to_number};
use crate::utils::codec::{BytesReader, FromLeBytes, Decode};
use crate::pvm::isa::skip;
use isa::no_arg::*;
use isa::two_reg::*;
use isa::two_reg_one_imm::*;
use isa::two_reg_one_offset::*;
use isa::three_reg::*;
use isa::one_reg_one_ext_imm::*;
use isa::two_imm::*;
use isa::one_reg_one_imm::*;
use isa::one_reg_two_imm::*;
use isa::one_reg_one_imm_one_offset::*;

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

    pvm_ctx.gas -= 1;

    let next_instruction = program.code[pvm_ctx.pc as usize];
    println!("next instruction = {next_instruction}");
    
    let exit_reason = match next_instruction { 

        0_u8    => { trap() },
        1_u8    => { fallthrough() },
        //7_u8    => { halt() },
        20_u8   => { load_imm_64(pvm_ctx, program) },
        30_u8   => { store_imm_u8(pvm_ctx, program) },
        31_u8   => { store_imm_u16(pvm_ctx, program) },
        32_u8   => { store_imm_u32(pvm_ctx, program) },
        33_u8   => { store_imm_u64(pvm_ctx, program) },
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
        141_u8  => { neg_add_imm_32(pvm_ctx, program) },
        147_u8  => { cmov_iz_imm(pvm_ctx, program) },
        149_u8  => { add_imm_64(pvm_ctx, program) },
        150_u8  => { mul_imm_64(pvm_ctx, program) },    
        154_u8  => { neg_add_imm_64(pvm_ctx, program) },
        170_u8  => { branch_eq(pvm_ctx, program) },
        171_u8  => { branch_ne(pvm_ctx, program) },
        172_u8  => { branch_lt_u(pvm_ctx, program) },
        173_u8  => { branch_lt_s(pvm_ctx, program) },
        174_u8  => { branch_ge_u(pvm_ctx, program) },
        175_u8  => { branch_ge_s(pvm_ctx, program) },
        190_u8  => { add_32(pvm_ctx, program) },
        191_u8  => { sub_32(pvm_ctx, program) },
        192_u8  => { mul_32(pvm_ctx, program) },
        193_u8  => { div_u_32(pvm_ctx, program) },
        194_u8  => { div_s_32(pvm_ctx, program) },
        196_u8  => { rem_s_32(pvm_ctx, program) },
        200_u8  => { add_64(pvm_ctx, program) },
        201_u8  => { sub_64(pvm_ctx, program) },
        202_u8  => { mul_64(pvm_ctx, program) },
        203_u8  => { div_u_64(pvm_ctx, program) },
        204_u8  => { div_s_64(pvm_ctx, program) },

        210_u8  => { and(pvm_ctx, program) },
        211_u8  => { xor(pvm_ctx, program) },
        212_u8  => { or(pvm_ctx, program) },
        218_u8  => { cmov_iz(pvm_ctx, program) },
        _       => { return ExitReason::trap },   // Panic
    };

    return exit_reason;
}

/*fn split_program_blob(blob: &[u8]) -> (Vec<u8>, Vec<bool>, Vec<u8>) {
    
    let mut program_blob = BytesReader::new(blob);

    let jump_table_size = decode_unsigned(&mut program_blob).unwrap();   // Dynamic jump table size
    let jump_opcode_size = program_blob.read_byte().unwrap();
    let program_code_size = decode_unsigned(&mut program_blob).unwrap(); // Program size
    
    let mut jump_table = vec![];  
    for i in 0..jump_table_size {
        jump_table.push(program_blob.read_byte().unwrap());
    }

    let program_code = program_blob.read_bytes(program_code_size as usize).unwrap();
    let bitmask =  Vec::<u8>::decode(&mut program_blob).unwrap();

    

    
    let program_size = program[2] as u32;
    let code = program[3..3 + program_size as usize].to_vec();
    let bitmask = decode_to_bits(&program[3 + program_size as usize..program.len()].to_vec())
        .into_iter()
        .enumerate()
        .map(|(i, bit)| if i >= program_size as usize { true } else { bit })
        .chain(std::iter::repeat(true).take(25 - (8 - program_size as usize % 8)))
        .collect();
    let jump_table = vec![];
    (code, bitmask, jump_table)
}*/

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
                pvm_ctx.pc += skip(&pvm_ctx.pc, &program.bitmask) + 1;
                println!("pc = {}", pvm_ctx.pc);
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

