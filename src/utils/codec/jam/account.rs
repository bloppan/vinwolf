/*use crate::constants::EPOCH_LENGTH;
use crate::types::{Account, WorkPackageHash, WorkReport};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};*/

// Esto ya se hace en ServiceInfo
/*impl Encode for Account {
    
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.code_hash.encode_to(&mut blob);
        self.balance.encode_to(&mut blob);
        self.gas.encode_to(&mut blob);
        self.min_gas.encode_to(&mut blob);
        let mut num_bytes: u64 = 0;
        let mut num_items: u32 = 0;
        for item in self.storage.iter() {
            num_bytes += item.1.len() as u64;
            num_items += 1;
        }
        num_bytes.encode_to(&mut blob);
        num_items.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Account {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        let code_hash = WorkPackageHash::decode(blob)?;
        let balance = u64::decode(blob)?;
        let gas = u64::decode(blob)?;
        let min_gas = u64::decode(blob)?;
        let num_bytes = u64::decode(blob)?;
        let num_items = u32::decode(blob)?;
        let mut storage = Vec::new();
        for _ in 0..num_items {
            let key = WorkPackageHash::decode(blob)?;
            let value = blob.read_bytes(num_bytes as usize)?;
            storage.push((key, value));
        }

        Ok(Account {
            code_hash,
            balance,
            gas,
            min_gas,
            storage,
        })
    }
}*/