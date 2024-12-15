use serde::Deserialize;
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};
use crate::blockchain::block::header::Header;
use crate::blockchain::block::extrinsic::Extrinsic;

pub mod header;
pub mod extrinsic;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TicketEnvelope {
    pub signature: String,
    pub attempt: u8,
}

#[derive(Debug)]
pub struct Block {
    pub header: Header,
    pub extrinsic: Extrinsic,
}

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