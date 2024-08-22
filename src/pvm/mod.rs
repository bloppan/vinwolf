use frame_support::Deserialize;

const MOVE_REG: u8          = 82;
const ADD: u8               = 8;
const ADD_IMM: u8           = 2;
const AND: u8               = 23;
const AND_IMM: u8           = 18;
const BRANCH_EQ_IMM: u8     = 7;
const LOAD_IMM: u8          = 4;


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

fn move_reg(value: &[u8], reg: &mut [u32; 13]) {
    let a: u8 = value[0] & 0x0F;
    let dest: u8 = value[0] >> 4;
    reg[dest as usize] = reg[a as usize];
}

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

pub fn invoke_pvm(pvm_ctx: &mut PVM) -> String {
    
    let j = pvm_ctx.program[0];
    let z = pvm_ctx.program[1];
    let program_len = pvm_ctx.program[2] as u32;
    
    if pvm_ctx.gas < pvm_ctx.program[2] as i64 { return "no_gas".to_string() }

    pvm_ctx.gas -= 1;

    while (pvm_ctx.pc < program_len) && (pvm_ctx.gas > 0) {
        
        match pvm_ctx.program[3] {
            MOVE_REG => { move_reg(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 2; },
            ADD => { add(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 3; },
            ADD_IMM => { add_imm(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 3; },
            AND => { and(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 3; },
            AND_IMM => { and_imm(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 3; },
            LOAD_IMM => { load_imm(&pvm_ctx.program[4..], &mut pvm_ctx.reg); pvm_ctx.pc += 3; },
            0 => { println!("TRAP"); }, 
            _ => { println!("trap!");},
        };
        pvm_ctx.gas -= 1;
    }
    
    "trap".to_string()
}

