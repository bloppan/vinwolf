use std::default::Default;
use core::array::from_fn;

use crate::constants::ENTROPY_POOL_SIZE;
use crate::types::{OpaqueHash, Entropy, EntropyPool};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

//static ENTROPY_STATE: Lazy<Mutex<EntropyPool>> = Lazy::new(|| Mutex::new(EntropyPool::default()));

impl Default for EntropyPool {
    fn default() -> Self {
        EntropyPool(Box::new(from_fn(|_| Entropy::default())))
    }
}

impl Default for Entropy {
    fn default() -> Self {
        Entropy(OpaqueHash::default())
    }
}

impl Encode for Entropy {
    fn encode(&self) -> Vec<u8> {
        self.0.encode()
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Entropy {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Entropy(OpaqueHash::decode(reader)?))
    }
}

impl Encode for EntropyPool {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(size_of::<Self>());
        
        for entropy in self.0.iter() {
            entropy.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for EntropyPool {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        let mut entropy_pool = EntropyPool::default();

        for entropy in entropy_pool.0.iter_mut() {
            *entropy = Entropy::decode(reader)?;
        }

        return Ok(entropy_pool);
    }
}

/*pub fn set_entropy_state(post_state: &EntropyPool) {
    let mut state = ENTROPY_STATE.lock().unwrap();
    *state = post_state.clone();
}

pub fn get_entropy_state() -> EntropyPool {
    let state = ENTROPY_STATE.lock().unwrap(); 
    return state.clone();
}*/

