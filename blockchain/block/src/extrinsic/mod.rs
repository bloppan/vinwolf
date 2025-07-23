use codec::{Encode, Decode, BytesReader};
use jam_types::ReadError;
use crate::{Extrinsic, TicketsExtrinsic, DisputesExtrinsic, PreimagesExtrinsic, GuaranteesExtrinsic, AssurancesExtrinsic};
pub mod assurances;
pub mod disputes;
pub mod preimages;
pub mod tickets;
pub mod guarantees;


impl Default for Extrinsic {
    fn default() -> Self {
        Extrinsic {
            tickets: TicketsExtrinsic::default(),
            disputes: DisputesExtrinsic::default(),
            preimages: PreimagesExtrinsic::default(),
            guarantees: GuaranteesExtrinsic::default(),
            assurances: AssurancesExtrinsic::default(),

        }
    }
}

impl Encode for Extrinsic {
    fn encode(&self) -> Vec<u8> {
        let mut extrinsic_blob: Vec<u8> = Vec::new();

        self.tickets.encode_to(&mut extrinsic_blob);
        self.preimages.encode_to(&mut extrinsic_blob);
        self.guarantees.encode_to(&mut extrinsic_blob);
        self.assurances.encode_to(&mut extrinsic_blob);
        self.disputes.encode_to(&mut extrinsic_blob);      

        return extrinsic_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}

impl Decode for Extrinsic {
    fn decode(extrinsic_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Extrinsic {
            tickets: TicketsExtrinsic::decode(extrinsic_blob)?,
            preimages: PreimagesExtrinsic::decode(extrinsic_blob)?,
            guarantees: GuaranteesExtrinsic::decode(extrinsic_blob)?,
            assurances: AssurancesExtrinsic::decode(extrinsic_blob)?,
            disputes: DisputesExtrinsic::decode(extrinsic_blob)?,
        })
    }
}

