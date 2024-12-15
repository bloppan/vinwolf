use crate::codec::{Encode, Decode, ReadError, BytesReader};
use crate::codec::tickets_extrinsic::TicketsExtrinsic;
use crate::codec::disputes_extrinsic::DisputesExtrinsic;
use crate::codec::preimages_extrinsic::PreimagesExtrinsic;
use crate::codec::assurances_extrinsic::AssurancesExtrinsic;
use crate::blockchain::block::extrinsic::guarantees::GuaranteesExtrinsic;

pub mod guarantees;
// The extrinsic data is split into its several portions:
//     Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring.
//     Votes, by validators, on dispute(s) arising between them presently taking place.
//     Static data which is presently being requested to be available for workloads to be able to fetch on demand.
//     Assurances by each validator concerning which of the input data of workloads they have correctly received and are 
//     storing locally.
//     Reports of newly completed workloads whose accuracy is guaranteed by specific validators.

#[derive(Debug)]
pub struct Extrinsic {
    pub tickets: TicketsExtrinsic,
    pub disputes: DisputesExtrinsic,
    pub preimages: PreimagesExtrinsic,
    pub assurances: AssurancesExtrinsic,
    pub guarantees: GuaranteesExtrinsic,
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

    pub fn encode(&self) -> Vec<u8> {
        let mut extrinsic_blob: Vec<u8> = Vec::new();

        self.tickets.encode_to(&mut extrinsic_blob);
        self.disputes.encode_to(&mut extrinsic_blob);
        self.preimages.encode_to(&mut extrinsic_blob);
        self.assurances.encode_to(&mut extrinsic_blob);
        self.guarantees.encode_to(&mut extrinsic_blob);

        return extrinsic_blob;
    }

    pub fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode()); 
    }
}
