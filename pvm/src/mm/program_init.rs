/*
    The software programs which will run in each of the four instances where the pvm is utilized in the main document have a 
    very typical setup pattern characteristic of an output of a compiler and linker. This means that ram has sections for 
    program-specific read-only data, read-write (heap) data and the stack. An adjunct to this, very typical of our usage patterns 
    is an extra read-only section via which invocation-specific data may be passed (i.e. arguments). It thus makes sense to define 
    this properly in a single initializer function. These sections are quantized into major zones, and one major zone is always 
    left unallocated between sections in order to reduce accidental overrun. Sections are padded with zeroes to the nearest pvm 
    memory page boundary.
*/

use crate::pvm_types::{RamAccess, RamAddress, RamMemory, Registers, StandardProgram, ProgramFormat, Page};
use jam_types::ReadError;
use constants::pvm::{Zi, Zz, NUM_PAGES, NUM_REG, PAGE_SIZE, PVM_INIT_ZONE_SIZE};
use codec::{Decode, DecodeSize, BytesReader};

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

        if 50 >= start_page && 50 <= end_page {
            //println!("ram page 50: {:x?}", self.pages[50].as_ref().unwrap().data);
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
               //println!("END: {:?}", end);
                self.curr_heap_pointer = page(end as usize) as RamAddress;
                //println!("init heap pointer: {:?}", self.curr_heap_pointer);
            },
            RamSection::Zone4 => {
                for page in start_page..=end_page {
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Write);
                    self.pages[page as usize].as_mut().unwrap().flags.access.insert(RamAccess::Read);
                }
                //println!("ram zone 4 page 50: {:x?}", self.pages[50].as_ref().unwrap().data);
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
            println!("Page {}: {:x?}", i, self.pages[i as usize].as_ref().unwrap().data);
        }*/

    }

}

use crate::{Program};
use codec::generic_codec::{decode_integer, decode_to_bits, decode_unsigned};

impl Decode for Program {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let jump_table_size = decode_unsigned(blob)?;   // Dynamic jump table size
        let jump_opcode_size = blob.read_byte()? as usize;   // Jump opcode size
        let program_code_size = decode_unsigned(blob)?; // Program size
        
        let mut jump_table = vec![];
    
        for _ in 0..jump_table_size {
            jump_table.push(decode_integer(blob, jump_opcode_size)?);
        }
        
        let program_code_slice = blob.read_bytes(program_code_size as usize)?;
        let code: Vec<u8> = program_code_slice.to_vec().into_iter().chain(std::iter::repeat(0).take(25)).collect();

        let num_bitmask_bytes = (program_code_size + 7) / 8;
        let mut bitmask = decode_to_bits(blob, num_bitmask_bytes as usize)?;
        bitmask.truncate(program_code_size);
        bitmask.extend(std::iter::repeat(true).take(code.len() - bitmask.len()));

        /*println!("\nProgram code len  = {} | Bitmask len = {}", program_code.len(), bitmask.len());
        println!("Jump table = {:?} \n", jump_table);
        println!("Program code = {:?}", program_code);
        println!("Bitmask = {:?}", bitmask);*/

        return Ok(Program {
            code,
            bitmask,
            jump_table,
        });
    }
}

impl Decode for ProgramFormat {
    
    fn decode(blob: &mut BytesReader) -> Result<ProgramFormat, ReadError> {
        
        let o_len = Vec::<u8>::decode_size(blob, 3)?;
        let w_len = Vec::<u8>::decode_size(blob, 3)?;
        let code_size = u16::decode(blob)?;
        let stack = Vec::<u8>::decode_size(blob, 3)?;
        let ro_data = blob.read_bytes(o_len as usize)?.to_vec();
        let rw_data = blob.read_bytes(w_len as usize)?.to_vec();
        let c_len = u32::decode(blob)?;
        let code = blob.read_bytes(c_len as usize)?.to_vec();
        
        /*println!("\no_len = {}", o_len);
        println!("w_len = {}", w_len);
        println!("z = {:?}", z);
        println!("s = {:?}", s);
        println!("o = {:?}", o);
        println!("w = {:?}", w);
        println!("c_len = {}\n", c_len);*/

        //println!("c = {:x?}", c);
        /*println!("Remaining bytes = {:?}", blob.get_position() - blob.data.len());
        println!("Program: ");
        for i in 0..20 {
            println!("{}", c[i]);
        }*/

        return Ok(ProgramFormat {
            code: code.to_vec(),
            ro_data: ro_data.to_vec(),
            rw_data: rw_data.to_vec(),
            code_size: code_size as u16,
            stack: stack as u32,
        });
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

