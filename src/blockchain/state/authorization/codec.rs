
use crate::types::{
    AuthPool, AuthPools, AuthQueue, AuthQueues, AuthorizerHash, AvailabilityAssignment, AvailabilityAssignments, AvailabilityAssignmentsItem, 
    OpaqueHash, WorkReport
};
use crate::constants::{CORES_COUNT, MAX_ITEMS_AUTHORIZATION_QUEUE};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError, encode_unsigned, decode_unsigned};

impl Encode for AvailabilityAssignment {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.report.encode_to(&mut blob);
        self.timeout.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignment {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(AvailabilityAssignment {
            report: WorkReport::decode(blob)?,
            timeout: u32::decode(blob)?,
        })
    }
}

impl Encode for AvailabilityAssignmentsItem {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<AvailabilityAssignment>());

        match self {
            None => {
                blob.push(0);
            }
            Some(assignment) => {
                blob.push(1);
                assignment.encode_to(&mut blob);
            }
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignmentsItem {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let option = blob.read_byte()?;
        match option {
            0 => Ok(None),
            1 => {
                let assignment = AvailabilityAssignment::decode(blob)?;
                Ok(Some(assignment))
            }
            _ => Err(ReadError::InvalidData),
        }
    }
}

impl Encode for AvailabilityAssignments {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<AvailabilityAssignmentsItem>() * CORES_COUNT);

        for assigment in self.assignments.iter() {
            assigment.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AvailabilityAssignments {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let mut assignments: AvailabilityAssignments = AvailabilityAssignments{assignments: Box::new(std::array::from_fn(|_| None))};
        
        for assignment in assignments.assignments.iter_mut() {
            *assignment = AvailabilityAssignmentsItem::decode(blob)?;
        }

        Ok(assignments)
    }
}

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
        let mut auth_pool = Vec::with_capacity(len);

        for _ in 0..len {
            auth_pool.push(OpaqueHash::decode(blob)?);
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

        let mut pools: AuthPools = AuthPools { auth_pools: Box::new(std::array::from_fn(|_| AuthPool { auth_pool: Vec::new() })) };

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