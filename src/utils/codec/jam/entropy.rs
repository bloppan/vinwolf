use crate::jam_types::{OpaqueHash, Entropy, EntropyPool};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};

impl Encode for Entropy {
    fn encode(&self) -> Vec<u8> {
        self.entropy.encode()
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Entropy {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Entropy {
            entropy: OpaqueHash::decode(reader)?,
        })
    }
}

impl Encode for EntropyPool {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(size_of::<Self>());
        
        for entropy in self.buf.iter() {
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

        for entropy in entropy_pool.buf.iter_mut() {
            *entropy = Entropy::decode(reader)?;
        }

        return Ok(entropy_pool);
    }
}

