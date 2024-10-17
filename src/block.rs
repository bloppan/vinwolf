use serde::Deserialize;
use crate::codec::{BytesReader, ReadError};
use crate::header::Header;
use crate::extrinsic::Extrinsic;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TicketEnvelope {
    pub signature: String,
    pub attempt: u8,
}

pub struct Block {
    header: Header,
    extrinsic: Extrinsic,
}

impl Block {

    pub fn decode(block_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let header = Header::decode(block_blob)?;
        let extrinsic = Extrinsic::decode(block_blob)?;
        Ok(Block { header, extrinsic })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut block_blob: Vec<u8> = Vec::new();
        self.header.encode_to(&mut block_blob);
        self.extrinsic.encode_to(&mut block_blob);
        return block_blob;
    }
}