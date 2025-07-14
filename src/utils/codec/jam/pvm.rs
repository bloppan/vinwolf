use crate::jam_types::{Program, ProgramFormat};
use crate::utils::codec::{Decode, DecodeSize, BytesReader, ReadError};
use crate::utils::codec::generic::{decode_integer, decode_to_bits, decode_unsigned};

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

