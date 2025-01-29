pub mod isa;
pub mod mm;

use std::collections::HashMap;
use std::u8;

use crate::constants::{NUM_REG, PAGE_SIZE};
use crate::types::{Context, ExitReason, PageFlags, Page, PageMap, Program, RamMemory, PageTable};
use crate::utils::codec::generic::{decode_to_bits, decode_unsigned, seq_to_number, decode_integer};
use crate::utils::codec::{BytesReader, FromLeBytes, Decode};
use crate::pvm::isa::skip;
use isa::one_offset::*;
use isa::no_arg::*;
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

impl Default for Context {
    fn default() -> Self {
        Context {
            pc: 0,
            gas: 0,
            reg: [0; NUM_REG as usize],
            page_table: PageTable::default(),
            page_fault: None,
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

pub fn invoke_pvm(pvm_ctx: &mut Context, program_blob: &[u8]) -> ExitReason {
    
    let program = {
        let (program_code, bitmask, jump_table) = split_program_blob(program_blob);
        Program {
            code: program_code,
            bitmask: bitmask,
            jump_table: jump_table,
        }
    };

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
                return ExitReason::page_fault;
            },
            _ => { return exit_reason; },
        }
    }

    return ExitReason::OutOfGas;
}

fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    pvm_ctx.gas -= 1;

    let next_instruction = program.code[pvm_ctx.pc as usize];
    println!("next instruction = {next_instruction}");
    
    let exit_reason = match next_instruction { 

        0_u8    => { trap() },
        1_u8    => { fallthrough() },
        //7_u8    => { halt() },
        //10_u8   => { ecalli(pvm_ctx, program) },
        20_u8   => { load_imm_64(pvm_ctx, program) },
        30_u8   => { store_imm_u8(pvm_ctx, program) },
        31_u8   => { store_imm_u16(pvm_ctx, program) },
        32_u8   => { store_imm_u32(pvm_ctx, program) },
        33_u8   => { store_imm_u64(pvm_ctx, program) },
        40_u8   => { jump(pvm_ctx, program) },
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
        80_u8   => { load_imm_jump(pvm_ctx, program) },
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
        120_u8  => { store_ind_u8(pvm_ctx, program) },
        121_u8  => { store_ind_u16(pvm_ctx, program) },
        122_u8  => { store_ind_u32(pvm_ctx, program) },
        123_u8  => { store_ind_u64(pvm_ctx, program) },
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
        136_u8  => { set_lt_u_imm(pvm_ctx, program) },
        137_u8  => { set_lt_s_imm(pvm_ctx, program) },
        138_u8  => { shlo_l_imm_32(pvm_ctx, program) },
        139_u8  => { shlo_r_imm_32(pvm_ctx, program) },
        140_u8  => { shar_r_imm_32(pvm_ctx, program) },
        141_u8  => { neg_add_imm_32(pvm_ctx, program) },
        142_u8  => { set_gt_u_imm(pvm_ctx, program) },
        143_u8  => { set_gt_s_imm(pvm_ctx, program) },
        144_u8  => { shlo_l_imm_alt_32(pvm_ctx, program) },
        145_u8  => { shlo_r_imm_alt_32(pvm_ctx, program) },
        146_u8  => { shar_r_imm_alt_32(pvm_ctx, program) },
        147_u8  => { cmov_iz_imm(pvm_ctx, program) },
        149_u8  => { add_imm_64(pvm_ctx, program) },
        150_u8  => { mul_imm_64(pvm_ctx, program) },
        151_u8  => { shlo_l_imm_64(pvm_ctx, program) },
        152_u8  => { shlo_r_imm_64(pvm_ctx, program) },
        153_u8  => { shar_r_imm_64(pvm_ctx, program) },    
        154_u8  => { neg_add_imm_64(pvm_ctx, program) },
        155_u8  => { shlo_l_imm_alt_64(pvm_ctx, program) },
        156_u8  => { shlo_r_imm_alt_64(pvm_ctx, program) },
        157_u8  => { shar_r_imm_alt_64(pvm_ctx, program) },
        170_u8  => { branch_eq(pvm_ctx, program) },
        171_u8  => { branch_ne(pvm_ctx, program) },
        172_u8  => { branch_lt_u(pvm_ctx, program) },
        173_u8  => { branch_lt_s(pvm_ctx, program) },
        174_u8  => { branch_ge_u(pvm_ctx, program) },
        175_u8  => { branch_ge_s(pvm_ctx, program) },
        180_u8  => { load_imm_jump_ind(pvm_ctx, program) },
        190_u8  => { add_32(pvm_ctx, program) },
        191_u8  => { sub_32(pvm_ctx, program) },
        192_u8  => { mul_32(pvm_ctx, program) },
        193_u8  => { div_u_32(pvm_ctx, program) },
        194_u8  => { div_s_32(pvm_ctx, program) },
        195_u8  => { rem_u_32(pvm_ctx, program) },
        196_u8  => { rem_s_32(pvm_ctx, program) },
        197_u8  => { shlo_l_32(pvm_ctx, program) },
        198_u8  => { shlo_r_32(pvm_ctx, program) },
        199_u8  => { shar_r_32(pvm_ctx, program) },
        200_u8  => { add_64(pvm_ctx, program) },
        201_u8  => { sub_64(pvm_ctx, program) },
        202_u8  => { mul_64(pvm_ctx, program) },
        203_u8  => { div_u_64(pvm_ctx, program) },
        204_u8  => { div_s_64(pvm_ctx, program) },
        205_u8  => { rem_u_64(pvm_ctx, program) },
        206_u8  => { rem_s_64(pvm_ctx, program) },
        207_u8  => { shlo_l_64(pvm_ctx, program) },
        208_u8  => { shlo_r_64(pvm_ctx, program) },
        209_u8  => { shar_r_64(pvm_ctx, program) },
        210_u8  => { and(pvm_ctx, program) },
        211_u8  => { xor(pvm_ctx, program) },
        212_u8  => { or(pvm_ctx, program) },
        216_u8  => { set_lt_u(pvm_ctx, program) },
        217_u8  => { set_lt_s(pvm_ctx, program) },
        218_u8  => { cmov_iz(pvm_ctx, program) },
        _       => { return ExitReason::trap },   // Panic
    };

    return exit_reason;
}


fn split_program_blob(blob: &[u8]) -> (Vec<u8>, Vec<bool>, Vec<usize>) {
    
    println!("Split program blob");
    let mut program_blob = BytesReader::new(blob);

    let jump_table_size = decode_unsigned(&mut program_blob).unwrap();   // Dynamic jump table size
    let jump_opcode_size = program_blob.read_byte().unwrap();
    let program_code_size = decode_unsigned(&mut program_blob).unwrap(); // Program size
    
    let mut jump_table = vec![];

    for _ in 0..jump_table_size {
        jump_table.push(decode_integer(&mut program_blob, jump_opcode_size as usize).unwrap());
    }
    
    let program_code_slice = program_blob.read_bytes(program_code_size as usize).unwrap();
    let program_code: Vec<u8> = program_code_slice.to_vec().into_iter().chain(std::iter::repeat(0).take(25)).collect();
    let current_pos = program_blob.get_position();

    let mut bitmask = decode_to_bits(&blob[current_pos..]);
    bitmask.truncate(program_code_size);
    bitmask.extend(std::iter::repeat(true).take(program_code.len() - bitmask.len()));


    println!("Program code len  = {} | Bitmask len = {}", program_code.len(), bitmask.len());
    println!("Jump table = {:?} \n", jump_table);
    println!("Program code = {:?}", program_code);
    println!("Bitmask = {:?}", bitmask);
    
    (program_code, bitmask, jump_table)
}



