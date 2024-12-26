use vinwolf::types::{TimeSlot, ValidatorIndex, Statistics, Extrinsic, ValidatorsData};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, PartialEq, Clone)]
pub struct InputStatistics {
    pub slot: TimeSlot,
    pub author_index: ValidatorIndex,
    pub extrinsic: Extrinsic
}

impl Encode for InputStatistics {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<InputStatistics>());

        self.slot.encode_to(&mut blob);
        self.author_index.encode_to(&mut blob);
        self.extrinsic.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputStatistics {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let slot = TimeSlot::decode(blob)?;
        let author_index = ValidatorIndex::decode(blob)?;
        let extrinsic = Extrinsic::decode(blob)?;

        Ok(InputStatistics {
            slot,
            author_index,
            extrinsic
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StateStatistics {
    pub pi: Statistics,
    pub tau: TimeSlot,
    pub kappa_prime: ValidatorsData
}

impl Encode for StateStatistics {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<StateStatistics>());

        self.pi.encode_to(&mut blob);
        self.tau.encode_to(&mut blob);
        self.kappa_prime.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateStatistics {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let pi = Statistics::decode(blob)?;
        let tau = TimeSlot::decode(blob)?;
        let kappa_prime = ValidatorsData::decode(blob)?;

        Ok(StateStatistics {
            pi,
            tau,
            kappa_prime
        })
    }
}

