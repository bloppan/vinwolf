pub mod isa;
pub mod mm;
pub mod hostcall;

use serde::Deserialize;
use std::collections::HashSet;
use codec::{BytesReader, Decode};
use jam_types::ReadError;

use isa::one_offset::*;
use isa::no_arg::*;
use isa::one_imm::*;
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
use constants::pvm::*;

pub type RamAddress = u32;
pub type PageAddress = RamAddress;
pub type PageNumber = u32;
pub type RegSize = u64;
pub type RegSigned = i64;
pub type Gas = i64;

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub pc: RegSize,
    pub gas: Gas,
    pub ram: RamMemory,
    pub reg: [RegSize; NUM_REG as usize],
    pub page_fault: Option<RamAddress>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RamMemory {
    pub pages: Box<[Option<Page>]>,
    pub curr_heap_pointer: RamAddress,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub flags: PageFlags,
    pub data: Box<[u8; PAGE_SIZE as usize]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageFlags {
    pub access: HashSet<RamAccess>,
    pub referenced: bool,
    pub modified: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub enum RamAccess {
    Read,
    Write,
    None,
}
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Program {
    pub code: Vec<u8>,          // Instruction data (c)
    pub bitmask: Vec<bool>,     // Bitmask (k)
    pub jump_table: Vec<usize>,    // Dynamic jump table (j)
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefineMemory {
    pub program: Vec<u8>,
    pub ram: RamMemory,
    pub pc: RegSize,
}

use std::convert::TryFrom;
impl TryFrom<u8> for HostCallFn {
    type Error = ReadError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0  => Ok(HostCallFn::Gas),
            1  => Ok(HostCallFn::Lookup),
            2  => Ok(HostCallFn::Read),
            3  => Ok(HostCallFn::Write),
            4  => Ok(HostCallFn::Info),
            5  => Ok(HostCallFn::Bless),
            6  => Ok(HostCallFn::Assign),
            7  => Ok(HostCallFn::Designate),
            8  => Ok(HostCallFn::Checkpoint),
            9  => Ok(HostCallFn::New),
            10 => Ok(HostCallFn::Upgrade),
            11 => Ok(HostCallFn::Transfer),
            12 => Ok(HostCallFn::Eject),
            13 => Ok(HostCallFn::Query),
            14 => Ok(HostCallFn::Solicit),
            15 => Ok(HostCallFn::Forget),
            16 => Ok(HostCallFn::Yield),
            17 => Ok(HostCallFn::HistoricalLookup),
            18 => Ok(HostCallFn::Fetch),
            19 => Ok(HostCallFn::Export),
            20 => Ok(HostCallFn::Machine),
            21 => Ok(HostCallFn::Peek),
            22 => Ok(HostCallFn::Poke),
            23 => Ok(HostCallFn::Zero),
            24 => Ok(HostCallFn::Void),
            25 => Ok(HostCallFn::Invoke),
            26 => Ok(HostCallFn::Expugne),
            27 => Ok(HostCallFn::Provide),
            100 => Ok(HostCallFn::Log),
            _  => Err(ReadError::InvalidData),
        }
    }
}

// ----------------------------------------------------------------------------------------------------------
// Host Call
// ----------------------------------------------------------------------------------------------------------
#[derive(Deserialize, Eq, Debug, Clone, PartialEq)]
pub enum HostCallFn {
    Gas = 0,
    Lookup = 1,
    Read = 2,
    Write = 3,
    Info = 4,
    Bless = 5,
    Assign = 6,
    Designate = 7,
    Checkpoint = 8,
    New = 9,
    Upgrade = 10,
    Transfer = 11,
    Eject = 12,
    Query = 13,
    Solicit = 14,
    Forget = 15,
    Yield = 16,
    HistoricalLookup = 17,
    Fetch = 18,
    Export = 19,
    Machine = 20,
    Peek = 21,
    Poke = 22,
    Zero = 23,
    Void = 24,
    Invoke = 25,
    Expugne = 26,
    Provide = 27,
    Log = 100,
    Unknown,
}

#[derive(Deserialize, Eq, Debug, Clone, PartialEq)]
pub enum HostCallError {
    InvalidContext,
    InvalidHostCall,
}

pub type Registers = [RegSize; NUM_REG as usize];

#[derive(Debug, Clone, PartialEq)]
pub struct StandardProgram {
    pub code: Vec<u8>,
    pub reg: [RegSize; NUM_REG],
    pub ram: RamMemory,
}

#[allow(unreachable_patterns)]
#[allow(non_snake_case)]
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ExitReason {
    #[allow(non_camel_case_types)]
    trap,
    #[allow(non_camel_case_types)]
    halt,
    Continue,
    Branch,
    #[serde(rename = "halt")]
    Halt,
    #[allow(non_camel_case_types)]
    panic,
    OutOfGas,
    #[allow(non_camel_case_types)]
    #[serde(rename = "page-fault")]
    page_fault,
    PageFault(u32),     
    HostCall(HostCallFn),      
}


#[derive(Debug, Clone, PartialEq)]
pub struct ProgramFormat {
    pub code: Vec<u8>,
    pub ro_data: Vec<u8>,
    pub rw_data: Vec<u8>,
    pub code_size: u16,
    pub stack: u32,
}

pub fn invoke_pvm(pvm_ctx: &mut Context, program_blob: &[u8]) -> ExitReason {

    log::debug!("Invoke inner pvm");

    let program = match Program::decode(&mut BytesReader::new(program_blob)) {
        Ok(program) => { program },
        Err(_) => { 
            log::error!("Panic: Decoding program");
            return ExitReason::panic; 
        }
    };

    let mut opcode_copy = program.code[pvm_ctx.pc.clone() as usize];

    loop {
        
        let exit_reason = single_step_pvm(pvm_ctx, &program);
        
        match exit_reason {
            ExitReason::Continue => {
                // continue
            },
            ExitReason::OutOfGas => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::error!("PVM: Out of gas!");
                return ExitReason::OutOfGas;
            },
            // TODO arreglar esto
            /*ExitReason::panic |*/ ExitReason::Halt => {
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::debug!("PVM: Halt");
                //pvm_ctx.pc = 0; // Esto pone en el GP que deberia ser 0 (con panic tambien) TODO
                return ExitReason::halt;
            },
            _ => { 
                log::debug!("Exit: pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
                log::debug!("PVM: {:?}", exit_reason);
                return exit_reason; 
            },
        }
       
        log::trace!("pc = {:?}, opcode = {:03?}, gas = {:?}, reg = {:?}", pvm_ctx.pc.clone(), opcode_copy, pvm_ctx.gas, pvm_ctx.reg);
        opcode_copy = program.code[pvm_ctx.pc.clone() as usize];
    }
}


fn single_step_pvm(pvm_ctx: &mut Context, program: &Program) -> ExitReason {

    pvm_ctx.gas -= 1;

    if pvm_ctx.gas < 0 {
        return ExitReason::OutOfGas;
    }

    let exit_reason = match program.code[pvm_ctx.pc as usize] { 

        TRAP                    => { trap() },
        FALLTHROUGH             => { fallthrough(pvm_ctx, program) },
        ECALLI                  => { ecalli(pvm_ctx, program) },
        LOAD_IMM_64             => { load_imm_64(pvm_ctx, program) },
        STORE_IMM_U8            => { store_imm_u8(pvm_ctx, program) },
        STORE_IMM_U16           => { store_imm_u16(pvm_ctx, program) },
        STORE_IMM_U32           => { store_imm_u32(pvm_ctx, program) },
        STORE_IMM_U64           => { store_imm_u64(pvm_ctx, program) },
        JUMP                    => { jump(pvm_ctx, program) },
        JUMP_IND                => { jump_ind(pvm_ctx, program) },
        LOAD_IMM                => { load_imm(pvm_ctx, program) },
        LOAD_U8                 => { load_u8(pvm_ctx, program) },
        LOAD_I8                 => { load_i8(pvm_ctx, program) },
        LOAD_U16                => { load_u16(pvm_ctx, program) },
        LOAD_I16                => { load_i16(pvm_ctx, program) },
        LOAD_U32                => { load_u32(pvm_ctx, program) },
        LOAD_I32                => { load_i32(pvm_ctx, program) },
        LOAD_U64                => { load_u64(pvm_ctx, program) },
        STORE_U8                => { store_u8(pvm_ctx, program) },
        STORE_U16               => { store_u16(pvm_ctx, program) },
        STORE_U32               => { store_u32(pvm_ctx, program) },
        STORE_U64               => { store_u64(pvm_ctx, program) },
        STORE_IMM_IND_U8        => { store_imm_ind_u8(pvm_ctx, program) },
        STORE_IMM_IND_U16       => { store_imm_ind_u16(pvm_ctx, program) },
        STORE_IMM_IND_U32       => { store_imm_ind_u32(pvm_ctx, program) },
        STORE_IMM_IND_U64       => { store_imm_ind_u64(pvm_ctx, program) },
        LOAD_IMM_JUMP           => { load_imm_jump(pvm_ctx, program) },
        BRANCH_EQ_IMM           => { branch_eq_imm(pvm_ctx, program) },
        BRANCH_NE_IMM           => { branch_ne_imm(pvm_ctx, program) },
        BRANCH_LT_U_IMM         => { branch_lt_u_imm(pvm_ctx, program) },
        BRANCH_LE_U_IMM         => { branch_le_u_imm(pvm_ctx, program) },
        BRANCH_GE_U_IMM         => { branch_ge_u_imm(pvm_ctx, program) },
        BRANCH_GT_U_IMM         => { branch_gt_u_imm(pvm_ctx, program) },
        BRANCH_LT_S_IMM         => { branch_lt_s_imm(pvm_ctx, program) },
        BRANCH_LE_S_IMM         => { branch_le_s_imm(pvm_ctx, program) },
        BRANCH_GE_S_IMM         => { branch_ge_s_imm(pvm_ctx, program) },
        BRANCH_GT_S_IMM         => { branch_gt_s_imm(pvm_ctx, program) },
        MOVE_REG                => { move_reg(pvm_ctx, program) }, 
        SBRK                    => { sbrk(pvm_ctx, program) },
        COUNT_SET_BITS_64       => { count_set_bits_64(pvm_ctx, program) },
        COUNT_SET_BITS_32       => { count_set_bits_32(pvm_ctx, program) },
        LEADING_ZERO_BITS_64    => { leading_zero_bits_64(pvm_ctx, program) },
        LEADING_ZERO_BITS_32    => { leading_zero_bits_32(pvm_ctx, program) },
        TRAILING_ZERO_BITS_64   => { trailing_zero_bits_64(pvm_ctx, program) },
        TRAILING_ZERO_BITS_32   => { trailing_zero_bits_32(pvm_ctx, program) },
        SIGN_EXTEND_8           => { sign_extend_8(pvm_ctx, program) },
        SIGN_EXTEND_16          => { sign_extend_16(pvm_ctx, program) },
        ZERO_EXTEND_16          => { zero_extend_16(pvm_ctx, program) },
        REVERSE_BYTES           => { reverse_bytes(pvm_ctx, program) },
        STORE_IND_U8            => { store_ind_u8(pvm_ctx, program) },
        STORE_IND_U16           => { store_ind_u16(pvm_ctx, program) },
        STORE_IND_U32           => { store_ind_u32(pvm_ctx, program) },
        STORE_IND_U64           => { store_ind_u64(pvm_ctx, program) },
        LOAD_IND_U8             => { load_ind_u8(pvm_ctx, program) },
        LOAD_IND_I8             => { load_ind_i8(pvm_ctx, program) },
        LOAD_IND_U16            => { load_ind_u16(pvm_ctx, program) },
        LOAD_IND_I16            => { load_ind_i16(pvm_ctx, program) },
        LOAD_IND_U32            => { load_ind_u32(pvm_ctx, program) },
        LOAD_IND_I32            => { load_ind_i32(pvm_ctx, program) },
        LOAD_IND_U64            => { load_ind_u64(pvm_ctx, program) },
        ADD_IMM_32              => { add_imm_32(pvm_ctx, program) }, 
        AND_IMM                 => { and_imm(pvm_ctx, program) },
        XOR_IMM                 => { xor_imm(pvm_ctx, program) },
        OR_IMM                  => { or_imm(pvm_ctx, program) },
        MUL_IMM_32              => { mul_imm_32(pvm_ctx, program) },
        SET_LT_U_IMM            => { set_lt_u_imm(pvm_ctx, program) },
        SET_LT_S_IMM            => { set_lt_s_imm(pvm_ctx, program) },
        SHLO_L_IMM_32           => { shlo_l_imm_32(pvm_ctx, program) },
        SHLO_R_IMM_32           => { shlo_r_imm_32(pvm_ctx, program) },
        SHAR_R_IMM_32           => { shar_r_imm_32(pvm_ctx, program) },
        NEG_ADD_IMM_32          => { neg_add_imm_32(pvm_ctx, program) },
        SET_GT_U_IMM            => { set_gt_u_imm(pvm_ctx, program) },
        SET_GT_S_IMM            => { set_gt_s_imm(pvm_ctx, program) },
        SHLO_L_IMM_ALT_32       => { shlo_l_imm_alt_32(pvm_ctx, program) },
        SHLO_R_IMM_ALT_32       => { shlo_r_imm_alt_32(pvm_ctx, program) },
        SHAR_R_IMM_ALT_32       => { shar_r_imm_alt_32(pvm_ctx, program) },
        CMOV_IZ_IMM             => { cmov_iz_imm(pvm_ctx, program) },
        CMOV_NZ_IMM             => { cmov_nz_imm(pvm_ctx, program) },
        ADD_IMM_64              => { add_imm_64(pvm_ctx, program) },
        MUL_IMM_64              => { mul_imm_64(pvm_ctx, program) },
        SHLO_L_IMM_64           => { shlo_l_imm_64(pvm_ctx, program) },
        SHLO_R_IMM_64           => { shlo_r_imm_64(pvm_ctx, program) },
        SHAR_R_IMM_64           => { shar_r_imm_64(pvm_ctx, program) },    
        NEG_ADD_IMM_64          => { neg_add_imm_64(pvm_ctx, program) },
        SHLO_L_IMM_ALT_64       => { shlo_l_imm_alt_64(pvm_ctx, program) },
        SHLO_R_IMM_ALT_64       => { shlo_r_imm_alt_64(pvm_ctx, program) },
        SHAR_R_IMM_ALT_64       => { shar_r_imm_alt_64(pvm_ctx, program) },
        ROT_R_64_IMM            => { rot_r_64_imm(pvm_ctx, program) },
        ROT_R_64_IMM_ALT        => { rot_r_64_imm_alt(pvm_ctx, program) },
        ROT_R_32_IMM            => { rot_r_32_imm(pvm_ctx, program) },
        ROT_R_32_IMM_ALT        => { rot_r_32_imm_alt(pvm_ctx, program) },
        BRANCH_EQ               => { branch_eq(pvm_ctx, program) },
        BRANCH_NE               => { branch_ne(pvm_ctx, program) },
        BRANCH_LT_U             => { branch_lt_u(pvm_ctx, program) },
        BRANCH_LT_S             => { branch_lt_s(pvm_ctx, program) },
        BRANCH_GE_U             => { branch_ge_u(pvm_ctx, program) },
        BRANCH_GE_S             => { branch_ge_s(pvm_ctx, program) },
        LOAD_IMM_JUMP_IND       => { load_imm_jump_ind(pvm_ctx, program) },
        ADD_32                  => { add_32(pvm_ctx, program) },
        SUB_32                  => { sub_32(pvm_ctx, program) },
        MUL_32                  => { mul_32(pvm_ctx, program) },
        DIV_U_32                => { div_u_32(pvm_ctx, program) },
        DIV_S_32                => { div_s_32(pvm_ctx, program) },
        REM_U_32                => { rem_u_32(pvm_ctx, program) },
        REM_S_32                => { rem_s_32(pvm_ctx, program) },
        SHLO_L_32               => { shlo_l_32(pvm_ctx, program) },
        SHLO_R_32               => { shlo_r_32(pvm_ctx, program) },
        SHAR_R_32               => { shar_r_32(pvm_ctx, program) },
        ADD_64                  => { add_64(pvm_ctx, program) },
        SUB_64                  => { sub_64(pvm_ctx, program) },
        MUL_64                  => { mul_64(pvm_ctx, program) },
        DIV_U_64                => { div_u_64(pvm_ctx, program) },
        DIV_S_64                => { div_s_64(pvm_ctx, program) },
        REM_U_64                => { rem_u_64(pvm_ctx, program) },
        REM_S_64                => { rem_s_64(pvm_ctx, program) },
        SHLO_L_64               => { shlo_l_64(pvm_ctx, program) },
        SHLO_R_64               => { shlo_r_64(pvm_ctx, program) },
        SHAR_R_64               => { shar_r_64(pvm_ctx, program) },
        AND                     => { and(pvm_ctx, program) },
        XOR                     => { xor(pvm_ctx, program) },
        OR                      => { or(pvm_ctx, program) },
        MUL_UPPER_S_S           => { mul_upper_s_s(pvm_ctx, program) },
        MUL_UPPER_U_U           => { mul_upper_u_u(pvm_ctx, program) },
        MUL_UPPER_S_U           => { mul_upper_s_u(pvm_ctx, program) },
        SET_LT_U                => { set_lt_u(pvm_ctx, program) },
        SET_LT_S                => { set_lt_s(pvm_ctx, program) },
        CMOV_IZ                 => { cmov_iz(pvm_ctx, program) },
        CMOV_NZ                 => { cmov_nz(pvm_ctx, program) },
        ROT_L_64                => { rot_l_64(pvm_ctx, program) },
        ROT_L_32                => { rot_l_32(pvm_ctx, program) },
        ROT_R_64                => { rot_r_64(pvm_ctx, program) },
        ROT_R_32                => { rot_r_32(pvm_ctx, program) },
        AND_INV                 => { and_inv(pvm_ctx, program) },
        OR_INV                  => { or_inv(pvm_ctx, program) },
        XNOR                    => { xnor(pvm_ctx, program) },
        MAX                     => { max(pvm_ctx, program) },
        MAX_U                   => { max_u(pvm_ctx, program) },
        MIN                     => { min(pvm_ctx, program) },
        MIN_U                   => { min_u(pvm_ctx, program) },
        _                       => { println!("Unknown instruction!"); return ExitReason::panic },
    };
    //println!("pc = {:?}, opcode = {:?}, reg = {:?}", pvm_ctx.pc, program.code[pvm_ctx.pc as usize], pvm_ctx.reg);
    return exit_reason;
}

impl Default for RamMemory {
    fn default() -> Self {
        let mut v: Vec<Option<Page>> = Vec::with_capacity(NUM_PAGES as usize);
        for _ in 0..NUM_PAGES {
            v.push(None);
        }
        RamMemory {
            pages: v.into_boxed_slice(),
            curr_heap_pointer: 0,
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
            access: HashSet::new(),
            referenced: false,
            modified: false,
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Context {
            pc: 0,
            gas: 0,
            reg: [0; NUM_REG as usize],
            ram: RamMemory::default(),
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