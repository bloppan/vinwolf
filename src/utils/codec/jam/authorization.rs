use std::collections::VecDeque;
use crate::types::{AuthPool, AuthPools, AuthQueue, AuthQueues, OpaqueHash, CodeAuthorizer, CodeAuthorizers, CoreIndex, Authorizer};
use crate::utils::codec::{BytesReader, Decode, DecodeLen, Encode, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for AuthPool {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>() * self.auth_pool.len());
        encode_unsigned(self.auth_pool.len()).encode_to(&mut blob);      
        
        for item in &self.auth_pool {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>)  {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AuthPool {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let len = decode_unsigned(blob)?;
        let mut auth_pool = VecDeque::with_capacity(len);

        for _ in 0..len {
            auth_pool.push_back(OpaqueHash::decode(blob)?);
        }

        Ok(AuthPool{
            auth_pool: auth_pool,
        })
    }
}

impl Encode for AuthPools {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<AuthPool>() * self.auth_pools.len());

        for item in self.auth_pools.iter() {
            item.encode_to(&mut blob);
        }
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AuthPools {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut pools: AuthPools = AuthPools::default();

        for pool in pools.auth_pools.iter_mut() {
            *pool = AuthPool::decode(blob)?;
        }

        Ok(AuthPools{
            auth_pools: pools.auth_pools,
        })
    }
}

impl Encode for AuthQueue { 

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        for item in self.auth_queue.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AuthQueue {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut queue = AuthQueue::default();

        for auth in queue.auth_queue.iter_mut() {
            *auth = OpaqueHash::decode(blob)?;
        }

        Ok(AuthQueue{
            auth_queue: queue.auth_queue,
        })
    }
}

impl Encode for AuthQueues {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<AuthQueue>() * self.auth_queues.len());

        for item in self.auth_queues.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AuthQueues {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut queues = AuthQueues::default();

        for queue in queues.auth_queues.iter_mut() {
            *queue = AuthQueue::decode(blob)?;
        }

        Ok(AuthQueues{
            auth_queues: queues.auth_queues,
        })
    }
}

impl Encode for CodeAuthorizer {

    fn encode(&self) -> Vec<u8> {

        let mut authorizer = Vec::with_capacity(std::mem::size_of::<Self>());

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

impl Encode for CodeAuthorizers {

    fn encode(&self) -> Vec<u8> {

        let mut authorizers = Vec::with_capacity(std::mem::size_of::<CodeAuthorizers>() * self.authorizers.len());
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

        let len = decode_unsigned(blob)?;
        let mut authorizers = Vec::with_capacity(len);

        for _ in 0..len {
            authorizers.push(CodeAuthorizer::decode(blob)?);
        }

        Ok(CodeAuthorizers {
            authorizers,
        })
    }
}

impl Encode for Authorizer {
    
    fn encode(&self) -> Vec<u8> {

        let mut authorizer = Vec::with_capacity(std::mem::size_of::<Self>());

        self.code_hash.encode_to(&mut authorizer);
        encode_unsigned(self.params.len()).encode_to(&mut authorizer);
        self.params.encode_to(&mut authorizer);

        return authorizer;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Authorizer {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Authorizer {
            code_hash: OpaqueHash::decode(blob)?,
            params: Vec::<u8>::decode_len(blob)?,
        })
    }
}