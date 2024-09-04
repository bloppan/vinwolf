use frame_support::Deserialize;

use super::codec;

//const RAM_SIZE: usize = 4 * 1024 * 1024 * 1024; // 4GB

/*
const MOVE_REG: u8          = 82;
const ADD: u8               = 8;
const ADD_IMM: u8           = 2;
const AND: u8               = 23;
const AND_IMM: u8           = 18;
const BRANCH_EQ_IMM: u8     = 7;
const LOAD_IMM: u8          = 4;
*/

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct MemoryChunk {
    address: u32,
    contents: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PageMap {
    address: u32,
    length: u32,
    is_writable: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[warn(non_camel_case_types)]
pub enum ExpectedStatus {
    Trap,
    Halt,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct PVM {
    pub reg: [u32; 13],
    pub pc: u32,
    pub page_map: Vec<PageMap>,    
    pub memory: Vec<PageMap>,
    pub gas: i64,
    pub program: Vec<u8>,
}



/*fn move_reg(pvm: &mut PVM) {
    let a: u8 = pvm.program[pvm.pc as usize + 1] & 0x0F;
    let dest: u8 = pvm.program[pvm.pc as usize + 1] >> 4;
    pvm.reg[dest as usize] = pvm.reg[a as usize];
    pvm.pc += 2;
}*/


fn add(value: &[u8], reg: &mut [u32; 13]) {
    let a: u8 = value[0] & 0x0F;
    let b: u8 = value[0] >> 4;
    let dest: u8 = value[1] & 0x0F;
    reg[dest as usize] = reg[a as usize].wrapping_add(reg[b as usize]);
}

fn add_imm(value: &[u8], reg: &mut [u32; 13]) {
    let b: u8 = value[0] >> 4;
    let dest: u8 = value[0] & 0x0F;
    reg[dest as usize] = reg[b as usize].wrapping_add(value[1] as u32);
}

fn and(value: &[u8], reg: &mut [u32; 13]) {
    let a: u8 = value[0] & 0x0F;
    let b: u8 = value[0] >> 4;
    let dest: u8 = value[1] & 0x0F;
    reg[dest as usize] = reg[a as usize] & reg[b as usize];
}

fn and_imm(value: &[u8], reg: &mut [u32; 13]) {
    let b: u8 = value[0] >> 4;
    let dest: u8 = value[0] & 0x0F;
    reg[dest as usize] = reg[b as usize] & value[1] as u32;
}

fn load_imm(prog_data: &[u8], reg: &mut [u32; 13]) {
    let dest = prog_data[0] as usize;
    println!("dest = {dest}");
    if dest >= reg.len() {
        panic!("Index out of bounds: dest = {}", dest);
    }
    let value: u16 = ((prog_data[2] as u16) << 8) | prog_data[1] as u16;
    println!("value = {value}");
    reg[dest as usize] = value as u32;
}

#[allow(non_camel_case_types)]
enum ExitReason {
    out_of_gas,
    halt,
    panic,
    host_call,
    page_fault,
}

/*fn single_step_pvm(
    c: Vec<u8>, // Instruction data
    j: Vec<u8>, // Dynamic jump table
    i: u32,     // Index instruction opcode
    gas: i64,   // Gas
    reg: [u8; 13], // Registers
    ram: Vec<u8>,  // Ram memory
) -> (String, u32, i64, [u8; 13], Vec<u8>) {


}*/

/*fn trap() -> String {

}*/
fn move_reg(value: u8, reg: &mut [u8; 13]) -> Result<(), String> {
    let a: u8 = value & 0x0F;
    let dest: u8 = value >> 4;
    if dest > 13 {return Err("panic".to_string())};
    reg[dest as usize] = reg[a as usize];
    Ok(())
}
fn trap() -> Result<(), String> {
    return Err("trap".to_string());
}

pub fn invoke_pvm(
    p: Vec<u8>, // Program blob
    mut i: u32,     // Index instruction opcode
    mut gas: i64,   // Gas
    mut reg: [u8; 13], // Registers
    mut ram: Vec<PageMap>,   // Ram memory 
) ->  (String, u32, i64, [u8; 13], Vec<PageMap>) { // Exit, i, gas, reg, ram
    
    let mut pc = i; // Program counter
    let j = p[0]; // Dynamic jump table size
    let z = p[1]; // Jump octects length
    let program_size = p[2] as u32;
    let instruction_data: Vec<u8> = save_chunk(p[3..].to_vec(), program_size)
        .into_iter()
        .chain(std::iter::repeat(0).take(24)) // Sequence of zeroes suffixed to ensure that no out-of-bounds access is possible
        .collect(); 
    let num_k_octets = number_of_octets(program_size as isize);
    let k_index: usize = (3 + program_size).try_into().unwrap(); // Index first bitmask octet
    let k_int = save_chunk(p[k_index..p.len()].to_vec(), program_size); // Bitmask integer vector
    //println!("k_int = {:?}", k_int);
    let k = codec::serialize_bits(k_int); // Bitmask boolean vector
    //println!("k = {:?}", k);

    while gas > 0 {
        let result = match instruction_data[i as usize] {
            82_u8 => { move_reg(instruction_data[i as usize + 1], &mut reg) },
            /*8 => { add(&pvm.program[4..], &mut pvm.reg); pvm.pc += 3; },
            2 => { add_imm(&pvm.program[4..], &mut pvm.reg); pvm.pc += 3; },
            23 => { and(&pvm.program[4..], &mut pvm.reg); pvm.pc += 3; },
            18 => { and_imm(&pvm.program[4..], &mut pvm.reg); pvm.pc += 3; },
            4 => { load_imm(&pvm.program[4..], &mut pvm.reg); pvm.pc += 3; },*/
            0 => { trap() },
            _ => Ok({ println!("No instruction!");}),
        };
        gas -= 1;
        // Check if there was an error and return the error message
        if let Err(err) = result {
            return (err, pc, gas, reg, ram);
        }
        i = skip(i, k.clone()) + 1;
        pc = i - 1;
        println!("next instruction = {}", instruction_data[i as usize]);
    }
    return (("out_of_gas".to_string()), pc, gas, reg, ram);
}

fn save_chunk(blob: Vec<u8>, size: u32) -> Vec<u8> {
    let mut chunk = vec![];
    for i in 0..blob.len() {
        chunk.push(blob[i as usize]);
    }
    chunk
}

fn skip(i: u32, k: Vec<bool>) -> u32 {
    let mut next_i = i + 1;
    while k[next_i as usize] == false {
        next_i += 1;
    }
    std::cmp::min(24, next_i) 
}

fn number_of_octets(value: isize) -> usize {
    let mut octets = 1;
    let mut val = value;
    while val > 255 {
        val >>= 8; 
        octets += 1;  
    }
    octets
}