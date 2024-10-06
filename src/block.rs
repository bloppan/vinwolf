use serde::Deserialize;
use crate::codec::*;
use crate::header::Header;
use crate::extrinsic::*;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TicketEnvelope {
    pub signature: String,
    pub attempt: u8,
}
// E ≡ (ET ,EV ,EP ,EA,EG)
/*#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Extrinsic {
    pub tickets: Vec<TicketEnvelope>, // Tickets
//    ev: String, // Votes
//    ep: String, // Preimages
//    ea: String, // Availability
//    eg: String, // Reports
//    e: Vec<u8>, // Extrinsic vector serialized
}*/

/*Extrinsic ::= SEQUENCE {
    tickets TicketsExtrinsic,
    disputes DisputesExtrinsic,
    preimages PreimagesExtrinsic,
    assurances AssurancesExtrinsic,
    guarantees GuaranteesExtrinsic
}*/


pub struct Extrinsic {
    tickets: TicketsExtrinsic,
    disputes: DisputesExtrinsic,
    preimages: PreimagesExtrinsic,
    assurances: AssurancesExtrinsic,
    guarantees: GuaranteesExtrinsic,
}

impl Extrinsic {
    pub fn decode(extrinsic_blob: &mut BytesReader) -> Result<Self, ReadError> {
        let tickets = TicketsExtrinsic::decode(extrinsic_blob)?;
        let disputes = DisputesExtrinsic::decode(extrinsic_blob)?;
        let preimages = PreimagesExtrinsic::decode(extrinsic_blob)?;
        let assurances = AssurancesExtrinsic::decode(extrinsic_blob)?;
        let guarantees = GuaranteesExtrinsic::decode(extrinsic_blob)?;

        Ok(Extrinsic {
            tickets,
            disputes,
            preimages,
            assurances,
            guarantees,
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut extrinsic_blob: Vec<u8> = Vec::new();
        self.tickets.encode_to(&mut extrinsic_blob)?;
        self.disputes.encode_to(&mut extrinsic_blob)?;
        self.preimages.encode_to(&mut extrinsic_blob)?;
        self.assurances.encode_to(&mut extrinsic_blob)?;
        self.guarantees.encode_to(&mut extrinsic_blob)?;

        Ok(extrinsic_blob)
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) -> Result<(), ReadError> {
        into.extend_from_slice(&self.encode()?); 
        Ok(())
    }
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

    pub fn encode(&self) -> Result<Vec<u8>, ReadError> {
        let mut block_blob: Vec<u8> = Vec::new();
        self.header.encode_to(&mut block_blob)?;
        self.extrinsic.encode_to(&mut block_blob)?;
        Ok(block_blob)
    }

}