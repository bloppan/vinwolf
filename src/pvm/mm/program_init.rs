/*
    The software programs which will run in each of the four instances where the pvm is utilized in the main document have a 
    very typical setup pattern characteristic of an output of a compiler and linker. This means that ram has sections for 
    program-specific read-only data, read-write (heap) data and the stack. An adjunct to this, very typical of our usage patterns 
    is an extra read-only section via which invocation-specific data may be passed (i.e. arguments). It thus makes sense to define 
    this properly in a single initializer function. These sections are quantized into major zones, and one major zone is always 
    left unallocated between sections in order to reduce accidental overrun. Sections are padded with zeroes to the nearest pvm 
    memory page boundary.
*/

use crate::types::{Page, ProgramFormat, RamAccess, RamAddress, RamMemory, Registers, StandardProgram};
use crate::constants::{Zi, Zz, NUM_PAGES, NUM_REG, PAGE_SIZE, PVM_INIT_ZONE_SIZE};
use crate::utils::codec::{Decode, BytesReader, ReadError};

pub fn page(x: usize) -> u64 {
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

/// We thus define the standard program code format p, which includes not only the instructions and jump table (previously represented 
/// by the term c), but also information on the state of the ram at program start. Given some p which is appropriately encoded together 
/// with some argument data a, we can define program code c, registers ω and ram µ through the standard initialization decoder function.
pub fn init_std_program(program: &[u8], arg: &[u8]) -> Result<Option<StandardProgram>, ReadError> {

    let mut blob = BytesReader::new(program);
    let params = ProgramFormat::decode(&mut blob)?;

    if 5 * Zz + zone(params.ro_data.len()) + zone(params.rw_data.len() + params.code_size as usize * PAGE_SIZE as usize) + zone(params.stack as usize) + Zi > (1 << 32) {
        return Ok(None);
    }

    let mut ram = RamMemory::default();
    ram.init(&params, arg);

    return Ok(Some(StandardProgram {
        ram,
        reg: init_registers(&params, arg),
        code: params.code,
    }));
}

pub fn init_registers(_params: &ProgramFormat, arg: &[u8]) -> Registers {

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

impl RamMemory {

    pub fn init(&mut self, params: &ProgramFormat, arg: &[u8]) {

            let start_zone_1 = Zz;
            let end_zone_1 = Zz + params.ro_data.len() as u64;
            self.init_section(params, arg, start_zone_1 as RamAddress, end_zone_1 as RamAddress, RamSection::Zone1);

            let start_zone_2 = Zz + params.ro_data.len() as u64;
            let end_zone_2 = Zz + page(params.ro_data.len()) as u64;
            self.init_section(params, arg, start_zone_2 as RamAddress, end_zone_2 as RamAddress, RamSection::Zone2);

            let start_zone_3 = 2 * Zz + zone(params.ro_data.len()) as u64;
            let end_zone_3 = 2 * Zz + zone(params.ro_data.len()) as u64 + params.rw_data.len() as u64;
            self.init_section(params, arg, start_zone_3 as RamAddress, end_zone_3 as RamAddress, RamSection::Zone3);

            let start_zone_4 = 2 * Zz + zone(params.ro_data.len()) as u64 + params.rw_data.len() as u64;
            let end_zone_4 = 2 * Zz + zone(params.ro_data.len()) as u64 + page(params.rw_data.len()) as u64 + (params.code_size as u64 * PAGE_SIZE as u64);
            self.init_section(params, arg, start_zone_4 as RamAddress, end_zone_4 as RamAddress, RamSection::Zone4);

            let start_zone_5 = (1 << 32) - 2 * Zz - Zi - page(params.stack as usize) as u64;
            let end_zone_5 = (1 << 32) - 2 * Zz - Zi;
            self.init_section(params, arg, start_zone_5 as RamAddress, end_zone_5 as RamAddress, RamSection::Zone5);

            let start_zone_6 = (1 << 32) - Zz - Zi;
            let end_zone_6 = (1 << 32) - Zz - Zi + arg.len() as u64;
            self.init_section(params, arg, start_zone_6 as RamAddress, end_zone_6 as RamAddress, RamSection::Zone6);

            let start_zone_7 = (1 << 32) - Zz - Zi + arg.len() as u64;
            let end_zone_7 = (1 << 32) - Zz - Zi + page(arg.len() as usize) as u64;
            self.init_section(params, arg, start_zone_7 as RamAddress, end_zone_7 as RamAddress, RamSection::Zone7);

    }

    fn init_section(&mut self,
        params: &ProgramFormat, 
        arg: &[u8],
        start: RamAddress, 
        end: RamAddress, 
        section: RamSection) 
    {
        let start_page = start / PAGE_SIZE;
        let end_page = (end - 1) / PAGE_SIZE;

        for i in start_page..=(end_page % NUM_PAGES) {
            if self.pages[(i % NUM_PAGES) as usize].is_none() {
                self.pages[(i % NUM_PAGES) as usize] = Some(Page::default());
            }
        }
        //println!("Initializing RAM section: {:?} | Start: {} | End: {}", section, start, end);
        match section {
            RamSection::Zone1 => {
                for i in start..end {
                    let page = i / PAGE_SIZE;
                    let offset = i % PAGE_SIZE;
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                    self.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.ro_data[i as usize - Zz as usize];
                }
            },
            RamSection::Zone2 => {
                for page in start_page..=end_page {
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                }
            },
            RamSection::Zone3 => {
                for i in start..end {
                    let page = i / PAGE_SIZE;
                    let offset = i % PAGE_SIZE;
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                    self.pages[page as usize].as_mut().unwrap().data[offset as usize] = params.rw_data[i as usize - (2 * Zz + zone(params.ro_data.len()) as u64) as usize];
                }
                self.curr_heap_pointer = end + PAGE_SIZE;
            },
            RamSection::Zone4 => {
                for page in start_page..=end_page {
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                }
            },
            RamSection::Zone5 => {
                for page in start_page..=end_page {
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                }
            },
            RamSection::Zone6 => {
                for i in start..end {
                    let page = i / PAGE_SIZE;
                    let offset = i % PAGE_SIZE;
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                    self.pages[page as usize].as_mut().unwrap().data[offset as usize] = arg[i as usize - ((1 << 32) - Zz - Zi) as usize]; 
                }
            },
            RamSection::Zone7 => {
                for page in start_page..=end_page {
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                }
            },
        }

        /*for i in start_page..=end_page {
            println!("Page {}: {:x?}", i, ram.pages[i as usize].as_ref().unwrap().data);
        }*/

    }

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

