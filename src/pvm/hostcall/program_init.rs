use crate::types::{Page, PageTable, ProgramFormat, RamAccess, RamAddress, RamMemory, StandardProgram};
use crate::constants::{NUM_REG, PAGE_SIZE, PVM_INIT_INPUT_DATA_SIZE, PVM_INIT_ZONE_SIZE, RAM_SIZE, Zz, Zi};
use crate::constants::{NONE, WHAT, OOB, WHO, FULL, CORE, CASH, LOW, HUH, OK};
use crate::utils::codec::{Decode, BytesReader, ReadError};

fn Page(x: usize) -> u64 {
    x.div_ceil(PAGE_SIZE as usize) as u64 * PAGE_SIZE as u64
}

fn Zone(x: usize) -> u64 {
    x.div_ceil(PVM_INIT_ZONE_SIZE as usize) as u64 * PVM_INIT_ZONE_SIZE as u64
}

enum RamSection {
    Zone1,
    Zone2,
    Zone3,
    Zone4,
    Zone5,
    Zone6,
    Zone7,
}

fn init_ram_section(ram: &mut RamMemory,
                    params: &ProgramFormat, 
                    arg: &[u8],
                    start: RamAddress, 
                    end: RamAddress, 
                    section: RamSection) 
{
    let start_page = start / PAGE_SIZE;
    let end_page = (end - 1) / PAGE_SIZE;

    for i in start_page..=end_page {
        if ram.pages[i as usize].is_none() {
            ram.pages[i as usize] = Some(Page::default());
        }
    }

    match section {
        RamSection::Zone1 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                let offset = i % PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.o[i as usize - Zz as usize];
            }
        },
        RamSection::Zone2 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                let offset = i % PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = 0;
            }
        },
        RamSection::Zone3 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                let offset = i % PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.w[i as usize - (2 * Zz + Zone(params.o.len()) as u64) as usize];
            }
        },
        RamSection::Zone4 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
            }
        },
        RamSection::Zone5 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
            }
        },
        RamSection::Zone6 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                let offset = i % PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = arg[i as usize - ((1 << 32) - Zz - Zi) as usize]; 
            }
        },
        RamSection::Zone7 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
            }
        },
    }
}


pub fn init_ram(params: &ProgramFormat, arg: &[u8]) -> RamMemory {

    let mut ram = RamMemory::default();
    
    let start_zone_1 = Zz;
    let end_zone_1 = Zz + params.o.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_1 as RamAddress, end_zone_1 as RamAddress, RamSection::Zone1);

    let start_zone_2 = Zz + params.o.len() as u64;
    let end_zone_2 = Zz + Page(params.o.len()) as u64;
    init_ram_section(&mut ram, params, arg, start_zone_2 as RamAddress, end_zone_2 as RamAddress, RamSection::Zone2);

    let start_zone_3 = 2 * Zz + Zone(params.o.len()) as u64;
    let end_zone_3 = 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_3 as RamAddress, end_zone_3 as RamAddress, RamSection::Zone3);

    let start_zone_4 = 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64;
    let end_zone_4 = 2 * Zz + Zone(params.o.len()) as u64 + Page(params.w.len()) as u64 + params.z as u64 * PAGE_SIZE as u64;
    init_ram_section(&mut ram, params, arg, start_zone_4 as RamAddress, end_zone_4 as RamAddress, RamSection::Zone4);

    let start_zone_5 = (1 << 32) - 2 * Zz - Zi - Page(params.s as usize) as u64;
    let end_zone_5 = (1 << 32) - 2 * Zz - Zi;
    init_ram_section(&mut ram, params, arg, start_zone_5 as RamAddress, end_zone_5 as RamAddress, RamSection::Zone5);

    let start_zone_6 = (1 << 32) - Zz - Zi;
    let end_zone_6 = (1 << 32) - Zz - Zi + arg.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_6 as RamAddress, end_zone_6 as RamAddress, RamSection::Zone6);

    let start_zone_7 = (1 << 32) - Zz - Zi + arg.len() as u64;
    let end_zone_7 = (1 << 32) - Zz - Zi + Page(arg.len() as usize) as u64;
    init_ram_section(&mut ram, params, arg, start_zone_7 as RamAddress, end_zone_7 as RamAddress, RamSection::Zone7);

    return ram;

}

pub fn init_registers(params: &ProgramFormat, arg: &[u8]) -> [u64; NUM_REG] {

    let mut reg = [0; NUM_REG];
    println!("init registers");
    for i in 0..NUM_REG {

        if i == 0 {
            reg[i] = 0xFFFF0000;
        } else if i == 1 {
            reg[i] = (1 << 32) - 2 * Zz - Zi;
        } else if i == 7 {
            reg[i] = (1 << 32) - Zz - Zi;
        } else if i == 8 {
            reg[i] = arg.len() as u64;
        } else {
            reg[i] = 0;
        }
    }
    println!("registers init done");
    return reg;
}

pub fn init_std_program(program: &[u8], arg: &[u8]) -> Result<Option<StandardProgram>, ReadError> {

    let mut blob = BytesReader::new(program);
    let params = ProgramFormat::decode(&mut blob)?;

    if 5 * Zz + Zone(params.o.len()) + Zone(params.w.len() + params.z as usize * PAGE_SIZE as usize) + Zone(params.s as usize) + Zi > (1 << 32) {
        return Ok(None);
    }

    return Ok(Some(StandardProgram {
        ram: init_ram(&params, arg),
        reg: init_registers(&params, arg),
        code: params.c,
    }));
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_division() {
        assert_eq!(1, 1_u32.div_ceil(2));
        assert_eq!(1, 2_u32.div_ceil(5));
        assert_eq!(3, 5_u32.div_ceil(2));
    }
}