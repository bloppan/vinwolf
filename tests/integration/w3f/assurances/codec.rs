use vinwolf::types::{TimeSlot, ValidatorsData, HeaderHash, AvailabilityAssignments, AssurancesExtrinsic};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct InputAssurances {
    pub assurances: AssurancesExtrinsic,
    pub slot: TimeSlot,
    pub parent: HeaderHash
}

impl Encode for InputAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut input = Vec::with_capacity(std::mem::size_of::<InputAssurances>());
        self.assurances.encode_to(&mut input);
        self.slot.encode_to(&mut input);
        self.parent.encode_to(&mut input);
        return input;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(InputAssurances {
            assurances: AssurancesExtrinsic::decode(reader)?,
            slot: TimeSlot::decode(reader)?,
            parent: HeaderHash::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateAssurances {
    pub avail_assignments: AvailabilityAssignments,
    pub curr_validators: ValidatorsData
}

impl Encode for StateAssurances {
    fn encode(&self) -> Vec<u8> {
        let mut state = Vec::with_capacity(std::mem::size_of::<StateAssurances>());
        self.avail_assignments.encode_to(&mut state);
        self.curr_validators.encode_to(&mut state);
        return state;
    }
    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAssurances {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(StateAssurances {
            avail_assignments: AvailabilityAssignments::decode(reader)?,
            curr_validators: ValidatorsData::decode(reader)?,
        })
    }
}


/*pub struct TestCase {
    pub input: Input,
    pub pre_state: State,
    pub output: Output,
    pub post_state: State
}*/
