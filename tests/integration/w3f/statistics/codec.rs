use vinwolf::types::{ValidatorStatistics, TimeSlot, ValidatorIndex, Extrinsic, ValidatorsData};
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
    pub curr_stats: ValidatorStatistics,
    pub prev_stats: ValidatorStatistics,
    pub tau: TimeSlot,
    pub curr_validators: ValidatorsData
}

impl Encode for StateStatistics {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.curr_stats.encode_to(&mut blob);
        self.prev_stats.encode_to(&mut blob);
        self.tau.encode_to(&mut blob);
        self.curr_validators.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateStatistics {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(StateStatistics {
            curr_stats: ValidatorStatistics::decode(blob)?,
            prev_stats: ValidatorStatistics::decode(blob)?,
            tau: TimeSlot::decode(blob)?,
            curr_validators: ValidatorsData::decode(blob)?
        })
    }
}

