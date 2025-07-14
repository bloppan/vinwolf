use crate::jam_types::{Header, Extrinsic, Block};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

pub mod header;
pub mod extrinsic;

impl Encode for Block {

    fn encode(&self) -> Vec<u8> {

        let mut block_blob: Vec<u8> = Vec::new();

        self.header.encode_to(&mut block_blob);
        self.extrinsic.encode_to(&mut block_blob);

        return block_blob;
    }
    
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    } 
}

impl Decode for Block {

    fn decode(block_blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let header = Header::decode(block_blob)?;
        let extrinsic = Extrinsic::decode(block_blob)?;

        Ok(Block { header, extrinsic })
    }
}