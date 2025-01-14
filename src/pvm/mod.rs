pub mod isa;

use frame_support::Deserialize;
use crate::constants::NUM_REG;
use crate::types::{PageMap, ProgramSequence, Context, ExitReason};
use crate::utils::codec::generic::{seq_to_number, decode_to_bits};
use isa::two_reg::*;

/*const NO_ARG: usize = 0;                     // Without arguments
const ONE_IMM: usize = 1;                    // Arguments of one immediate
const TWO_IMM: usize = 2;                    // Arguments of two immediate
const ONE_OFFSET: usize = 3;                 // Arguments of one offset
*/const ONE_REG_ONE_IMM: usize = 4;            // Arguments of one reg and one immediate
/*const ONE_REG_TWO_IMM: usize = 5;            // Arguments of one reg and two immediate
*/const ONE_REG_ONE_IMM_ONE_OFFSET: usize = 6; // Arguments of one reg, one immediate and one offset
/*const TWO_REG: usize = 7;                    // Arguments of two regs
*/const TWO_REG_ONE_IMM: usize = 8;            // Arguments of two regs and one immediate
/*const TWO_REG_ONE_OFFSET: usize = 9;         // Arguments of two regs and one offset
const TWO_REG_TWO_IMM: usize = 10;           // Arguments of two regs and two immediates
const THREE_REG: usize = 11;                 // Arguments of three regs
*/

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

fn trap() -> Result<(), String> {
    return Err("trap".to_string());
}

fn panic() -> Result<(), String> {
    return Err("panic".to_string());
}

fn skip(i: &mut usize, k: &Vec<bool>) {
    let mut j = *i + 1;
    //println!("k = {:?}", k);
    while j < k.len() && k[j] == false {
        j += 1;
    }
    //println!("j = {}", j-1);
    *i = std::cmp::min(24, j - 1);
}


/*

fn extend_sign(v: &Vec<u8>, n: u32) -> u32 {
    let x = seq_to_number(v);
    if n == 0 { return x; }
    let sign_bit = (x / (1u32 << (8 * n - 1))) as u64;
    return x + ((sign_bit * ((1u64 << 32) - (1u64 << (8 * n)))) as u32);
}

fn get_imm(program: &ProgramSequence, pc: u32, instr_type: usize) -> u32 {
    let mut i = pc; 
    let l_x = match instr_type {
        ONE_REG_ONE_IMM | 
        TWO_REG_ONE_IMM => { 
                            i += 2; 
                            //println!("TWO_REG_ONE_IMM");
                            let x: isize = skip(pc, &program.k).saturating_sub(1) as isize;
                            let x_u32 = if x < 0 { 0 } else { x as u32 }; 
                            std::cmp::min(4_u32, x_u32)
                        },
        ONE_REG_ONE_IMM_ONE_OFFSET => {
                            i += 2;
                            //println!("ONE_REG_ONE_IMM_ONE_OFFSET");
                            std::cmp::min(4_u32, (program.c[pc as usize + 1] / 16) as u32)
        },
        _ => return 0,
    };
    //println!("lx = {l_x}");
    return extend_sign(&program.c[i as usize ..i as usize + l_x as usize].to_vec(), (4 - l_x as usize) as u32);
}

*/

/*fn basic_block_seq(pc: usize, k: &Vec<bool>) -> u32 {
    return 1 + skip(pc, k) as u32;
}*/

fn single_step_pvm(pvm_ctx: &mut Context, program: &ProgramSequence) 
    -> Result<(), ExitReason> {

    //println!("next instruction = {}", program.c[pvm_ctx.pc as usize]);
    let result = match program.data[pvm_ctx.pc] {  // Next instruction
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
    pc: usize,    // Program counter
    gas: i64,   // Gas
    reg: [u64; NUM_REG as usize],     // Registers
    ram: Vec<PageMap>,  // Ram memory 
) ->  (ExitReason, usize, i64, [u64; NUM_REG as usize], Vec<PageMap>) { // Exit, i, gas, reg, ram
    
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