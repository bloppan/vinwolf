use std::collections::{HashSet, HashMap};
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
    pub reg: Registers,
    pub page_fault: Option<RamAddress>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RamMemory {
    pub pages: HashMap<PageNumber, Page>,
    pub curr_heap_pointer: RamAddress,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub flags: PageFlags,
    pub data: Box<[u8; PAGE_SIZE as usize]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageFlags {
    pub read_access: bool,
    pub write_access: bool,
    pub referenced: bool,
    pub modified: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum RamAccess {
    Read,
    Write,
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
            1  => Ok(HostCallFn::Fetch),
            2  => Ok(HostCallFn::Lookup),
            3  => Ok(HostCallFn::Read),
            4  => Ok(HostCallFn::Write),
            5  => Ok(HostCallFn::Info),
            6  => Ok(HostCallFn::HistoricalLookup),
            7  => Ok(HostCallFn::Export),
            8  => Ok(HostCallFn::Machine),
            9  => Ok(HostCallFn::Peek),
            10 => Ok(HostCallFn::Poke),
            11 => Ok(HostCallFn::Pages),
            12 => Ok(HostCallFn::Invoke),
            13 => Ok(HostCallFn::Expugne),
            14 => Ok(HostCallFn::Bless),
            15 => Ok(HostCallFn::Assign),
            16 => Ok(HostCallFn::Designate),
            17 => Ok(HostCallFn::Checkpoint),
            18 => Ok(HostCallFn::New),
            19 => Ok(HostCallFn::Upgrade),
            20 => Ok(HostCallFn::Transfer),
            21 => Ok(HostCallFn::Eject),
            22 => Ok(HostCallFn::Query),
            23 => Ok(HostCallFn::Solicit),
            24 => Ok(HostCallFn::Forget),
            25 => Ok(HostCallFn::Yield),
            26 => Ok(HostCallFn::Provide),
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
    Fetch = 1,
    Lookup = 2,
    Read = 3,
    Write = 4,
    Info = 5,
    HistoricalLookup = 6,
    Export = 7,
    Machine = 8,
    Peek = 9,
    Poke = 10,
    Pages = 11,
    Invoke = 12,
    Expugne = 13,
    Bless = 14,
    Assign = 15,
    Designate = 16,
    Checkpoint = 17,
    New = 18,
    Upgrade = 19,
    Transfer = 20,
    Eject = 21,
    Query = 22,
    Solicit = 23,
    Forget = 24,
    Yield = 25,
    Provide = 26,
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
/*impl Default for RamMemory {
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
}*/
impl Default for RamMemory {
    fn default() -> Self {
        RamMemory { pages: HashMap::new(), curr_heap_pointer: 0 }
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
            read_access: false,
            write_access: false,
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