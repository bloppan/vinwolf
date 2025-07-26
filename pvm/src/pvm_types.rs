use std::collections::HashSet;
use serde::Deserialize;

use constants::pvm::{NUM_REG, PAGE_SIZE, NUM_PAGES};
use jam_types::ReadError;

pub type RamAddress = u32;
pub type PageAddress = RamAddress;
pub type PageNumber = u32;
pub type RegSize = u64;
pub type RegSigned = i64;
pub type Gas = i64;
pub type Registers = [RegSize; NUM_REG as usize];

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

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramFormat {
    pub code: Vec<u8>,
    pub ro_data: Vec<u8>,
    pub rw_data: Vec<u8>,
    pub code_size: u16,
    pub stack: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StandardProgram {
    pub code: Vec<u8>,
    pub reg: [RegSize; NUM_REG],
    pub ram: RamMemory,
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
// ----------------------------------------------------------------------------------------------------------
// Default
// ----------------------------------------------------------------------------------------------------------
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