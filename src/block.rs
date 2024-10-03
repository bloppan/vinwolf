use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TicketEnvelope {
    pub signature: String,
    pub attempt: u8,
}
// E ≡ (ET ,EV ,EP ,EA,EG)
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Extrinsic {
    pub tickets: Vec<TicketEnvelope>, // Tickets
//    ev: String, // Votes
//    ep: String, // Preimages
//    ea: String, // Availability
//    eg: String, // Reports
//    e: Vec<u8>, // Extrinsic vector serialized
}

/*use crate::types::*;
use crate::globals::*;

use crate::codec::*;
*/
/*
use crate::header::Header;
use crate::extrinsic::Extrinsic;


pub struct Block {
    header: Header,
    extrinsic: Extrinsic,
}

impl Block {

    pub fn decode(block_blob: &[u8]) -> Result<Self, ReadError> {
        let mut blob = SliceReader::new(block_blob);
        let header = Header::decode(blob);
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {

        Ok(vec![])
    }

}*/