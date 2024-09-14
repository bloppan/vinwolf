use frame_support::Deserialize;

use super::codec;


const NO_ARG: usize = 0;                     // Without arguments
const ONE_IMM: usize = 1;                    // Arguments of one immediate
const TWO_IMM: usize = 2;                    // Arguments of two immediate
const ONE_OFFSET: usize = 3;                 // Arguments of one offset
const ONE_REG_ONE_IMM: usize = 4;            // Arguments of one reg and one immediate
const ONE_REG_TWO_IMM: usize = 5;            // Arguments of one reg and two immediate
const ONE_REG_ONE_IMM_ONE_OFFSET: usize = 6; // Arguments of one reg, one immediate and one offset
const TWO_REG: usize = 7;                    // Arguments of two regs
const TWO_REG_ONE_IMM: usize = 8;            // Arguments of two regs and one immediate
const TWO_REG_ONE_OFFSET: usize = 9;         // Arguments of two regs and one offset
const TWO_REG_TWO_IMM: usize = 10;           // Arguments of two regs and two immediates
const THREE_REG: usize = 11;                 // Arguments of three regs

/*#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MemoryChunk {
    address: u32,
    contents: Vec<u8>,
}*/

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PVM {
    pub pc: u32,
    pub gas: i64,
    pub ram: Vec<PageMap>,
    pub reg: [u32; 13],
}

struct ProgramSequence {
    c: Vec<u8>,   // Instrucción c
    k: Vec<bool>, // Bitmask k
    j: Vec<u8>,   // Dynamic jump table
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PageMap {
    address: u32,
    length: u32,
    is_writable: bool,
}

fn trap() -> Result<(), String> {
    return Err("trap".to_string());
}

fn panic() -> Result<(), String> {
    return Err("panic".to_string());
}

fn skip(i: u32, k: &Vec<bool>) -> u32 {
    let mut j = i + 1;
    //println!("k = {:?}", k);
    while j < k.len() as u32 && k[j as usize] == false {
        j += 1;
    }
    //println!("j = {}", j-1);
    std::cmp::min(24, j - 1)
}

fn move_reg(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs -> 52 00 -> r0 = r0
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn add_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs one imm -> 02 79 02 | r9 = r7 + 0x2
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, TWO_REG_ONE_IMM);
    //println!("value = {value}");
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize].wrapping_add(value);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn extend_sign(v: &Vec<u8>, n: u32) -> u32 {
    let x = codec::seq_to_number(v);
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

fn add(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 08 87 09 | r9 = r7 + r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) }; 
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize].wrapping_add(pvm_ctx.reg[b as usize]);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and(pvm_ctx: &mut PVM, program: &ProgramSequence) // Three regs -> 17 87 09 | r9 = r7 & r8
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize] & pvm_ctx.reg[b as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // Two regs one imm -> 12 79 03 | r9 = r7 & 0x3
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    let value = get_imm(program, pvm_ctx.pc, TWO_REG_ONE_IMM);
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize] & value;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn load_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // One reg one imm -> 04 07 d2 04 | r7 = 0x4d2
    -> Result<(), String> {
    let dest = program.c[pvm_ctx.pc as usize + 1];
    if dest > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, ONE_REG_ONE_IMM);
    pvm_ctx.reg[dest as usize] = value as u32;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);   
    Ok(())
}

fn branch_eq_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) // One reg, one imm, one offset -> 07 27 d3 04 | jump if r7 = 0x4d3
    -> Result<(), String> {
    let target = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if target > 13 { return Err("panic".to_string()) };
    let value = get_imm(program, pvm_ctx.pc, ONE_REG_ONE_IMM_ONE_OFFSET);
    //println!("value branch = {value}");
    if pvm_ctx.reg[target as usize] == value as u32 {
        pvm_ctx.pc = basic_block_seq(pvm_ctx.pc, &program.k);
    } 
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn basic_block_seq(pc: u32, k: &Vec<bool>) -> u32 {
    return 1 + skip(pc, k) as u32;
}

fn single_step_pvm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {

    //println!("next instruction = {}", program.c[pvm_ctx.pc as usize]);
    let result = match program.c[pvm_ctx.pc as usize] {  // Next instruction
        82_u8   => { move_reg(pvm_ctx, program) }, 
        8_u8    => { add(pvm_ctx, program) },
        2_u8    => { add_imm(pvm_ctx, program) },
        23_u8   => { and(pvm_ctx, program) },
        18_u8   => { and_imm(pvm_ctx, program) },
        4_u8    => { load_imm(pvm_ctx, program) },
        7_u8    => { branch_eq_imm(pvm_ctx, program) },
        0_u8    => { trap() },    // Trap
        _       => { panic() },   // Panic
    };
    pvm_ctx.gas -= 1;
    
    return result;
}

pub fn invoke_pvm(
    p: Vec<u8>,     // Program blob
    pc: u32,    // Program counter
    gas: i64,   // Gas
    reg: [u32; 13],     // Registers
    ram: Vec<PageMap>,  // Ram memory 
) ->  (String, u32, i64, [u32; 13], Vec<PageMap>) { // Exit, i, gas, reg, ram
    
    let j_size = p[0];          // Dynamic jump table size
    let j: Vec<u8> = vec![];    // Dynamic jump table
    let z = p[1];               // Jump octects length
    let program_size = p[2] as u32;

    let program = ProgramSequence {
        c: p[3..3 + program_size as usize].to_vec() // Instruction vector
            .into_iter()
            .chain(std::iter::repeat(0).take(25)) // Sequence of zeroes suffixed to ensure that no out-of-bounds access is possible
            .collect(),
        k: codec::serialize_bits(p[3 + program_size as usize..p.len()].to_vec())
            .into_iter()
            .chain(std::iter::repeat(true).take(25)) // Sequence of trues 
            .collect(), // Bitmask boolean vector
        j: j, // Dynamic jump table
    };
    /*println!("Program sequence = {:?}", program.c);
    println!("Bitmask sequence = {:?}", program.k);
    println!("Program len = {} Bitmask len = {}", program.c.len(), program.k.len());*/
    let mut pvm_ctx = PVM { pc: pc, gas: gas, reg: reg, ram: ram }; // PVM context
    let mut exit_reason;
    
    while gas > 0 {

        exit_reason = single_step_pvm(&mut pvm_ctx, &program);
        if let Err(err) = exit_reason {
            return (err, pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
        }
        pvm_ctx.pc += 1; // Next instruction
    }
    return (("out_of_gas".to_string()), pvm_ctx.pc, pvm_ctx.gas, pvm_ctx.reg, pvm_ctx.ram);
}