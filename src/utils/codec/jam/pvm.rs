use crate::types::Program;
use crate::utils::codec::{Decode, BytesReader, ReadError};
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
        let program_code: Vec<u8> = program_code_slice.to_vec().into_iter().chain(std::iter::repeat(0).take(25)).collect();

        let num_bitmask_bytes = (program_code_size + 7) / 8;
        let mut bitmask = decode_to_bits(blob, num_bitmask_bytes as usize)?;
        bitmask.truncate(program_code_size);
        bitmask.extend(std::iter::repeat(true).take(program_code.len() - bitmask.len()));

        /*println!("\nProgram code len  = {} | Bitmask len = {}", program_code.len(), bitmask.len());
        println!("Jump table = {:?} \n", jump_table);
        println!("Program code = {:?}", program_code);
        println!("Bitmask = {:?}", bitmask);*/

        return Ok(Program {
            code: program_code,
            bitmask: bitmask,
            jump_table: jump_table,
        });
    }
}