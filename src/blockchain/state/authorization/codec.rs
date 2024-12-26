use std::collections::VecDeque;
use crate::types::{
    AuthPool, AuthPools, AuthQueue, AuthQueues, AuthorizerHash, OpaqueHash, CodeAuthorizer, CodeAuthorizers, CoreIndex
};
use crate::constants::{CORES_COUNT, MAX_ITEMS_AUTHORIZATION_POOL, MAX_ITEMS_AUTHORIZATION_QUEUE};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError, encode_unsigned, decode_unsigned};

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
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

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

        let mut pools: AuthPools = AuthPools { auth_pools: Box::new(std::array::from_fn(|_| AuthPool { auth_pool: VecDeque::with_capacity(MAX_ITEMS_AUTHORIZATION_POOL) })) };

        for i in 0..CORES_COUNT {
            pools.auth_pools[i] = AuthPool::decode(blob)?;
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

        let mut auth_queue: Box<[AuthorizerHash; MAX_ITEMS_AUTHORIZATION_QUEUE]> = Box::new(std::array::from_fn(|_| [0; std::mem::size_of::<AuthorizerHash>()]));

        for i in 0..MAX_ITEMS_AUTHORIZATION_QUEUE {
            auth_queue[i] = OpaqueHash::decode(blob)?;
        }

        Ok(AuthQueue{
            auth_queue: auth_queue,
        })
    }
}

impl Encode for AuthQueues {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

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

        let mut queues: AuthQueues = AuthQueues { auth_queues: Box::new(std::array::from_fn(|_| AuthQueue { auth_queue: Box::new(std::array::from_fn(|_| [0; std::mem::size_of::<AuthorizerHash>()])) })) };

        for i in 0..CORES_COUNT {
            queues.auth_queues[i] = AuthQueue::decode(blob)?;
        }

        Ok(AuthQueues{
            auth_queues: queues.auth_queues,
        })
    }
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