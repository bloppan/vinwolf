use crate::types::{ProgramFormat, RamAddress, RamMemory, Page, StandardProgram};
use crate::constants::{NUM_REG, PAGE_SIZE, PVM_INIT_INPUT_DATA_SIZE, PVM_INIT_ZONE_SIZE, RAM_SIZE, Zz, Zi};
use crate::utils::codec::{Decode, BytesReader, ReadError};

fn Page(x: usize) -> u64 {
    x.div_ceil(PAGE_SIZE as usize) as u64 * PAGE_SIZE as u64
}

fn Zone(x: usize) -> u64 {
    x.div_ceil(PVM_INIT_ZONE_SIZE as usize) as u64 * PVM_INIT_ZONE_SIZE as u64
}

fn ram_initialization(params: &ProgramFormat, arg: &[u8]) -> RamMemory {

    let mut ram = RamMemory::default();

    for i in 0..RAM_SIZE {

        let page = i / PAGE_SIZE as u64;
        let offset = i % PAGE_SIZE as u64;

        if Zz <= i && i < Zz + params.o.len() as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.o[i as usize - Zz as usize];
        } else if Zz + params.o.len() as u64 <= i && i < Zz + Page(params.o.len()) as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = 0;
        } else if 2 * Zz + Zone(params.o.len()) as u64 <= i && i < 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.is_writable = true;
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.w[i as usize - (2 * Zz + Zone(params.o.len()) as u64) as usize];
        } else if 2 * Zz + Zone(params.o.len()) as u64 + params.w.len() as u64 <= i && i < 2 * Zz + Zone(params.o.len()) as u64 + Page(params.w.len()) as u64 + params.z as u64 * PAGE_SIZE as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.is_writable = true;
        } else if (1 << 32) - 2 * Zz - Zi - Page(params.s as usize) as u64 <= i && i < (1 << 32) - 2 * Zz - Zi {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().flags.is_writable = true;
        } else if (1 << 32) - Zz - Zi <= i && i < (1 << 32) - Zz - Zi + arg.len() as u64{
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
            ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = arg[i as usize - ((1 << 32) - Zz - Zi) as usize]; 
        } else if (1 << 32) - Zz - Zi + arg.len() as u64 <= i && i < (1 << 32) - Zz - Zi + Page(arg.len() as usize) as u64 {
            if ram.pages[page as usize].is_none() {
                ram.pages[page as usize] = Some(Page::default());
            }
        }
    }

    return ram;

}

fn reg_initialization(params: &ProgramFormat, arg: &[u8]) -> [u64; NUM_REG] {

    let mut reg = [0; NUM_REG];

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

    return reg;
}

fn standard_program_initialization(program: &[u8], arg: &[u8]) -> Result<Option<StandardProgram>, ReadError> {

    let mut blob = BytesReader::new(program);
    let params = ProgramFormat::decode(&mut blob)?;

    if 5 * Zz + Zone(params.o.len()) + Zone(params.w.len() + params.z as usize * PAGE_SIZE as usize) + Zone(params.s as usize) + Zi > (1 << 32) {
        return Ok(None);
    }

    return Ok(Some(StandardProgram {
        ram: ram_initialization(&params, arg),
        reg: reg_initialization(&params, arg),
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