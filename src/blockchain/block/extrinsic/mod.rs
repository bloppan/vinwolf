use crate::types::{TicketsExtrinsic, DisputesExtrinsic, PreimagesExtrinsic, AssurancesExtrinsic, GuaranteesExtrinsic, Extrinsic};
use crate::utils::codec::{Encode, Decode, ReadError, BytesReader};

pub mod assurances;
pub mod disputes;
pub mod preimages;
pub mod tickets;
pub mod guarantees;

// The extrinsic data is split into its several portions:
//     Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring.
//     Votes, by validators, on dispute(s) arising between them presently taking place.
//     Static data which is presently being requested to be available for workloads to be able to fetch on demand.
//     Assurances by each validator concerning which of the input data of workloads they have correctly received and are 
//     storing locally.
//     Reports of newly completed workloads whose accuracy is guaranteed by specific validators.

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

