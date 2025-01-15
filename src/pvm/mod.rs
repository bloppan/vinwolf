pub mod isa;

use std::u8;

use frame_support::Deserialize;
use crate::constants::NUM_REG;
use crate::types::{PageMap, ProgramSequence, Context, ExitReason};
use crate::utils::codec::generic::{seq_to_number, decode_to_bits};
use crate::utils::codec::FromLeBytes;
use isa::two_reg::*;

impl Default for Context {
    fn default() -> Self {
        Context {
            pc: 0,
            gas: 0,
            reg: [0; NUM_REG as usize],
            ram: vec![],
        }
    }
}

impl Default for ProgramSequence {
    fn default() -> Self {
        ProgramSequence {
            data: vec![],
            bitmask: vec![],
            jump_table: vec![],
        }
    }
}

fn single_step_pvm(pvm_ctx: &mut Context, program: &ProgramSequence) 
    -> Result<(), ExitReason> {

    //println!("next instruction = {}", program.c[pvm_ctx.pc as usize]);
    let result = match program.data[pvm_ctx.pc as usize] {  // Next instruction
        100_u8   => { move_reg(pvm_ctx, program)? }, 
        /*8_u8    => { add(pvm_ctx, program) },
        2_u8    => { add_imm(pvm_ctx, program) },
        23_u8   => { and(pvm_ctx, program) },
        18_u8   => { and_imm(pvm_ctx, program) },
        4_u8    => { load_imm(pvm_ctx, program) },
        7_u8    => { branch_eq_imm(pvm_ctx, program) },
        0_u8    => { trap() },    // Trap*/
        _       => { return Err(ExitReason::Panic) },   // Panic
    };
    pvm_ctx.gas -= 1;
    
    return Ok(result);
}

pub fn invoke_pvm(
    p: Vec<u8>,     // Program blob
    pc: u64,    // Program counter
    gas: i64,   // Gas
    reg: [u64; NUM_REG as usize],     // Registers
    ram: Vec<PageMap>,  // Ram memory 
) ->  (ExitReason, u64, i64, [u64; NUM_REG as usize], Vec<PageMap>) { // Exit, i, gas, reg, ram
    
    let _j_size = p[0];          // Dynamic jump table size
    let j: Vec<u8> = vec![];    // Dynamic jump table
    let _z = p[1];               // Jump octects length
    let program_size = p[2] as u32;

    let program = ProgramSequence {
        data: p[3..3 + program_size as usize].to_vec() // Instruction vector
            .into_iter()
            .chain(std::iter::repeat(0).take(25)) // Sequence of zeroes suffixed to ensure that no out-of-bounds access is possible
            .collect(),
        bitmask: decode_to_bits(&p[3 + program_size as usize..p.len()].to_vec())
            .into_iter()
            .chain(std::iter::repeat(true).take(25)) // Sequence of trues 
            .collect(), // Bitmask boolean vector
        jump_table: j, // Dynamic jump table
    };
    /*println!("Program sequence = {:?}", program.c);
    println!("Bitmask sequence = {:?}", program.k);
    println!("Program len = {} Bitmask len = {}", program.c.len(), program.k.len());*/
    let mut pvm_ctx = Context { pc, gas, reg, ram }; // PVM context
    let mut exit_reason;
    
    while gas > 0 {

        exit_reason = single_step_pvm(&mut pvm_ctx, &program);
        if let Err(err) = exit_reason {
            return (err, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
        }
        pvm_ctx.pc += 1; // Next instruction
    }
    return (ExitReason::OutOfGas, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
}

