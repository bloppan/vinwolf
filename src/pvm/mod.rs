use frame_support::Deserialize;

use super::codec;

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
    println!("k = {:?}", k);
    while j < k.len() as u32 && k[j as usize] == false {
        j += 1;
    }
    println!("j = {}", j-1);
    std::cmp::min(24, j - 1)
}

fn move_reg(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn add_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize].wrapping_add(program.c[pvm_ctx.pc as usize + 2] as u32);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn add(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) }; 
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize].wrapping_add(pvm_ctx.reg[b as usize]);
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 2] & 0x0F;
    if dest > 13 { return Err("panic".to_string()) };
    let a: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[a as usize] & pvm_ctx.reg[b as usize];
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn and_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest: u8 = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if dest > 13 {return Err("panic".to_string())};
    let b: u8 = program.c[pvm_ctx.pc as usize + 1] >> 4;
    pvm_ctx.reg[dest as usize] = pvm_ctx.reg[b as usize] & program.c[pvm_ctx.pc as usize + 2] as u32;
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn load_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let dest = program.c[pvm_ctx.pc as usize + 1];
    if dest > 13 { return Err("panic".to_string()) };
    let next_i = skip(pvm_ctx.pc, &program.k);
    let l_x = calc_l_x(pvm_ctx.pc, &program.k);
    let value = load(&program.c, l_x, pvm_ctx.pc);
    pvm_ctx.reg[dest as usize] = value as u32;
    pvm_ctx.pc = next_i;
    Ok(())
}

fn branch_eq_imm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {
    let l_x = calc_l_x(pvm_ctx.pc, &program.k);
    if l_x == 0 { return Err("panic".to_string()) };
    let target = program.c[pvm_ctx.pc as usize + 1] & 0x0F;
    if target > 13 { return Err("panic".to_string()) };
    let value = load(&program.c, l_x, pvm_ctx.pc);
    if pvm_ctx.reg[target as usize] == value as u32 {
        pvm_ctx.pc = basic_block_seq(pvm_ctx.pc, &program.k);
    } 
    pvm_ctx.pc = skip(pvm_ctx.pc, &program.k);
    Ok(())
}

fn calc_l_x(pc: u32, data: &Vec<bool>) -> u32 {
    std::cmp::min(4,std::cmp::max(0,skip(pc, data)-1-pc))
}

fn load(data: &Vec<u8>, l_x: u32, pc: u32) -> u32 {       
    if l_x == 1 {
        return data[pc as usize + 1] as u32;
    } else if l_x >= 2 && l_x <= 3 {
        println!("first");
        return codec::seq_to_number(&data[pc as usize + 2..pc as usize + 5].to_vec(), 2);
    } else {
        println!("second");
        return codec::seq_to_number(&data[pc as usize + 2..pc as usize + 6].to_vec(), 4);
    }
}

fn basic_block_seq(pc: u32, k: &Vec<bool>) -> u32 {
    return 1 + skip(pc, k) as u32;
}

fn single_step_pvm(pvm_ctx: &mut PVM, program: &ProgramSequence) 
    -> Result<(), String> {

    println!("next instruction = {}", program.c[pvm_ctx.pc as usize]);
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
    mut pc: u32,    // Program counter
    mut gas: i64,   // Gas
    mut reg: [u32; 13],     // Registers
    mut ram: Vec<PageMap>,  // Ram memory 
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
        k: codec::serialize_bits(p[3 + program_size as usize..p.len()].to_vec()), // Bitmask boolean vector
        j: j, // Dynamic jump table
    };

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