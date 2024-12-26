use vinwolf::types::{TimeSlot, AuthPools, AuthQueues, CodeAuthorizers};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct InputAuthorizations {
    pub slot: TimeSlot,
    pub auths: CodeAuthorizers,
}

impl Encode for InputAuthorizations {
    fn encode(&self) -> Vec<u8> {
        let mut input = Vec::with_capacity(std::mem::size_of::<InputAuthorizations>());
        self.slot.encode_to(&mut input);
        self.auths.encode_to(&mut input);
        return input;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAuthorizations {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputAuthorizations {
            slot: TimeSlot::decode(reader)?,
            auths: CodeAuthorizers::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateAuthorizations {
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
}

impl Encode for StateAuthorizations {
    fn encode(&self) -> Vec<u8> {
        let mut state = Vec::with_capacity(std::mem::size_of::<StateAuthorizations>());
        self.auth_pools.encode_to(&mut state);
        self.auth_queues.encode_to(&mut state);
        return state;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAuthorizations {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(StateAuthorizations {
            auth_pools: AuthPools::decode(blob)?,
            auth_queues: AuthQueues::decode(blob)?,
        })
    }
}