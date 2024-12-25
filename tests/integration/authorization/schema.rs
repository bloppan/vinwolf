use vinwolf::types::{TimeSlot, AuthPools, AuthQueues, CoreIndex, OpaqueHash};
use vinwolf::utils::codec::{encode_unsigned, decode_unsigned, BytesReader, Decode, Encode, ReadError};

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
pub struct CodeAuthorizer {
    pub core: CoreIndex,
    pub auth_hash: OpaqueHash,
}

impl Encode for CodeAuthorizer {
    fn encode(&self) -> Vec<u8> {
        let mut authorizer = Vec::with_capacity(std::mem::size_of::<CodeAuthorizer>());
        self.core.encode_to(&mut authorizer);
        self.auth_hash.encode_to(&mut authorizer);
        return authorizer;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for CodeAuthorizer {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(CodeAuthorizer {
            core: CoreIndex::decode(reader)?,
            auth_hash: OpaqueHash::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeAuthorizers {
    authorizers: Vec<CodeAuthorizer>,
}

impl Encode for CodeAuthorizers {
    fn encode(&self) -> Vec<u8> {
        let mut authorizers = Vec::with_capacity(std::mem::size_of::<CodeAuthorizers>());
        encode_unsigned(self.authorizers.len()).encode_to(&mut authorizers);
        for authorizer in &self.authorizers {
            authorizer.encode_to(&mut authorizers);
        }
        return authorizers;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for CodeAuthorizers {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        let mut authorizers = Vec::new();
        let len = decode_unsigned(blob)?;
        for _ in 0..len {
            authorizers.push(CodeAuthorizer::decode(blob)?);
        }
        Ok(CodeAuthorizers {
            authorizers,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateAuthorizations {
    auth_pools: AuthPools,
    auths_queues: AuthQueues,
}

impl Encode for StateAuthorizations {
    fn encode(&self) -> Vec<u8> {
        let mut state = Vec::with_capacity(std::mem::size_of::<StateAuthorizations>());
        self.auth_pools.encode_to(&mut state);
        self.auths_queues.encode_to(&mut state);
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
            auths_queues: AuthQueues::decode(blob)?,
        })
    }
}