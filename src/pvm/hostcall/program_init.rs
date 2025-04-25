use crate::types::{Page, ProgramFormat, RamAccess, RamAddress, RamMemory, Registers, StandardProgram};
use crate::constants::{NUM_REG, PAGE_SIZE, PVM_INIT_ZONE_SIZE, Zz, Zi};
use crate::utils::codec::{Decode, BytesReader, ReadError};

fn page(x: usize) -> u64 {
    x.div_ceil(PAGE_SIZE as usize) as u64 * PAGE_SIZE as u64
}

fn zone(x: usize) -> u64 {
    x.div_ceil(PVM_INIT_ZONE_SIZE as usize) as u64 * PVM_INIT_ZONE_SIZE as u64
}

#[derive(Debug, Clone, PartialEq)]
enum RamSection {
    Zone1,
    Zone2,
    Zone3,
    Zone4,
    Zone5,
    Zone6,
    Zone7,
}

fn init_ram_section(
    ram: &mut RamMemory,
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
    //println!("Initializing RAM section: {:?} | Start: {} | End: {}", section, start, end);
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
            for page in start_page..=end_page {
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
            }
        },
        RamSection::Zone3 => {
            for i in start..end {
                let page = i / PAGE_SIZE;
                let offset = i % PAGE_SIZE;
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                ram.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.w[i as usize - (2 * Zz + zone(params.o.len()) as u64) as usize];
            }
        },
        RamSection::Zone4 => {
            for page in start_page..=end_page {
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
            }
        },
        RamSection::Zone5 => {
            for page in start_page..=end_page {
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
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
            for page in start_page..=end_page {
                ram.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
            }
        },
    }

    /*for i in start_page..=end_page {
        println!("Page {}: {:x?}", i, ram.pages[i as usize].as_ref().unwrap().data);
    }*/

}


pub fn init_ram(params: &ProgramFormat, arg: &[u8]) -> RamMemory {

    let mut ram = RamMemory::default();
    
    let start_zone_1 = Zz;
    let end_zone_1 = Zz + params.o.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_1 as RamAddress, end_zone_1 as RamAddress, RamSection::Zone1);

    let start_zone_2 = Zz + params.o.len() as u64;
    let end_zone_2 = Zz + page(params.o.len()) as u64;
    init_ram_section(&mut ram, params, arg, start_zone_2 as RamAddress, end_zone_2 as RamAddress, RamSection::Zone2);

    let start_zone_3 = 2 * Zz + zone(params.o.len()) as u64;
    let end_zone_3 = 2 * Zz + zone(params.o.len()) as u64 + params.w.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_3 as RamAddress, end_zone_3 as RamAddress, RamSection::Zone3);

    let start_zone_4 = 2 * Zz + zone(params.o.len()) as u64 + params.w.len() as u64;
    let end_zone_4 = 2 * Zz + zone(params.o.len()) as u64 + page(params.w.len()) as u64 + (params.z as u64 * PAGE_SIZE as u64);
    init_ram_section(&mut ram, params, arg, start_zone_4 as RamAddress, end_zone_4 as RamAddress, RamSection::Zone4);

    let start_zone_5 = (1 << 32) - 2 * Zz - Zi - page(params.s as usize) as u64;
    let end_zone_5 = (1 << 32) - 2 * Zz - Zi;
    init_ram_section(&mut ram, params, arg, start_zone_5 as RamAddress, end_zone_5 as RamAddress, RamSection::Zone5);

    let start_zone_6 = (1 << 32) - Zz - Zi;
    let end_zone_6 = (1 << 32) - Zz - Zi + arg.len() as u64;
    init_ram_section(&mut ram, params, arg, start_zone_6 as RamAddress, end_zone_6 as RamAddress, RamSection::Zone6);

    let start_zone_7 = (1 << 32) - Zz - Zi + arg.len() as u64;
    let end_zone_7 = (1 << 32) - Zz - Zi + page(arg.len() as usize) as u64;
    init_ram_section(&mut ram, params, arg, start_zone_7 as RamAddress, end_zone_7 as RamAddress, RamSection::Zone7);

    return ram;

}

pub fn init_registers(_params: &ProgramFormat, arg: &[u8]) -> [u64; NUM_REG] {

    let mut reg = Registers::default();

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
    //println!("Registers: {:?}", reg);
    return reg;
}

pub fn init_std_program(program: &[u8], arg: &[u8]) -> Result<Option<StandardProgram>, ReadError> {

    let mut blob = BytesReader::new(program);
    let params = ProgramFormat::decode(&mut blob)?;

    if 5 * Zz + zone(params.o.len()) + zone(params.w.len() + params.z as usize * PAGE_SIZE as usize) + zone(params.s as usize) + Zi > (1 << 32) {
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
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;

    #[test]
    fn division_test() {
        assert_eq!(1, 1_u32.div_ceil(2));
        assert_eq!(1, 1_u32.div_ceil(3));
        assert_eq!(1, 2_u32.div_ceil(5));
        assert_eq!(3, 5_u32.div_ceil(2));
    }

    #[test]
    fn zone_page_test() {
        assert_eq!(4096, page(2000));
        assert_eq!(8192, page(5000));
        assert_eq!(131072, zone(100000));
    }

    fn read_test_file(filename: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(filename);
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => panic!("Failed to open file '{}': {}", path.display(), e),
        };
        let mut test_content = Vec::new();
        let _ = file.read_to_end(&mut test_content);
        test_content
    }

    #[test]
    fn bootstrap_test() {

        let bootstrap_blob = read_test_file(&format!("tests/services/bootstrap/bootstrap.pvm"));
        init_std_program(&bootstrap_blob, &[]).unwrap();

        let null_authorizer_blob = read_test_file(&format!("tests/services/null_authorizer/null_authorizer.pvm"));
        init_std_program(&null_authorizer_blob, &[]).unwrap();

        let blake2b_blob = read_test_file(&format!("tests/services/blake2b/blake2b.pvm"));
        init_std_program(&blake2b_blob, &[]).unwrap();
    }
}

