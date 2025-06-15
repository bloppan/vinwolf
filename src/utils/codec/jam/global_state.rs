use sp_core::blake2_256;
use crate::types::{OpaqueHash, StateKeyType, StateKey, KeyValue, StorageKey, RawState, StateRoot};
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, ReadError, BytesReader};

pub trait StateKeyTrait {
    fn construct(&self) -> StateKey;
}

impl StateKeyTrait for StateKeyType {
    fn construct(&self) -> StateKey {
        let mut key_result = StateKey::default();
        
        match self {
            StateKeyType::U8(value) => {
                key_result[..1].copy_from_slice(&value.encode());
            }
            StateKeyType::Service(value, service_id) => {
                key_result[..1].copy_from_slice(&value.encode());
                let service_encoded = u32::encode(service_id);
                key_result[1] = service_encoded[0];
                key_result[3] = service_encoded[1];
                key_result[5] = service_encoded[2];
                key_result[7] = service_encoded[3];
            }
            StateKeyType::Account(service_id, account) => {
                let service_encoded = u32::encode(service_id);
                let mut account_array = StateKey::default();
                account_array[..account.len().min(31)].copy_from_slice(&account[..account.len().min(31)]);
                key_result[0] = service_encoded[0];
                key_result[1] = account_array[0];
                key_result[2] = service_encoded[1];
                key_result[3] = account_array[1];
                key_result[4] = service_encoded[2];
                key_result[5] = account_array[2];
                key_result[6] = service_encoded[3];
                key_result[7] = account_array[3];
                key_result[8..].copy_from_slice(&account_array[4..27]);
            }
        }

        key_result
    }
}

pub fn construct_storage_key(key: &StorageKey) -> StateKey {
    let mut key_result = StorageKey::default();
    key_result[..4].copy_from_slice(&u32::MAX.encode());
    key_result[4..].copy_from_slice(&key[..27]);
    key_result
}

pub fn construct_preimage_key(hash: &OpaqueHash) -> StorageKey {
    let mut key_result = StorageKey::default();
    key_result[..4].copy_from_slice(&(u32::MAX - 1).encode());
    key_result[4..].copy_from_slice(&hash[1..28]);
    key_result
}

pub fn construct_lookup_key(hash: &OpaqueHash, length: u32) -> StorageKey {
    let mut key_result = StorageKey::default();
    key_result[..4].copy_from_slice(&length.encode());
    key_result[4..].copy_from_slice(&(blake2_256(hash)[2..29]));
    key_result
}

impl Encode for KeyValue {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.key.encode_to(&mut blob);
        self.value.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for KeyValue {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(KeyValue { key: StorageKey::decode(reader)?, value: Vec::<u8>::decode_len(reader)? })
    }
}

impl Encode for RawState {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.state_root.encode_to(&mut blob);
        self.keyvals.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RawState {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(RawState{
            state_root: StateRoot::decode(reader)?,
            keyvals: Vec::<KeyValue>::decode_len(reader)?,
        })    
    }
}


#[cfg(test)]
mod test {

    use crate::utils::codec::jam::global_state::StateKeyType;
    use super::*;

    #[test]
    fn test_state_key() {
        
        let result= StateKeyType::U8(5).construct();
        //let result = key.construct();
        println!("result = {:?} ", result);
        assert_eq!(1, 1);

        let key = StateKeyType::Service(2, 7);
        println!("Service = {:?}", key.construct());

        let key = StateKeyType::Account(4, vec![0, 1, 2]);
        println!("Account = {:?}", key.construct());

        let key = StateKeyType::Account(0xAABBCCDD, (0..=26).collect());
        println!("Account = {:?}", key.construct());

        let key = StateKeyType::Account(0xAABBCCDD, (0..=55).collect());
        println!("Account = {:?}", key.construct());
    }
}

