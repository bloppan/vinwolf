use jam_types::{KeyValue, StorageKey, RawState, StateRoot, *};
use crate::{Encode, EncodeLen, Decode, DecodeLen, ReadError, BytesReader};
use crate::generic_codec::*;
use constants::node::{
    AUTH_POOLS, AUTH_QUEUE, RECENT_HISTORY, SAFROLE, DISPUTES, ENTROPY, NEXT_VALIDATORS, CURR_VALIDATORS, PREV_VALIDATORS, AVAILABILITY, TIME, PRIVILEGES,
    STATISTICS, READY_QUEUE, ACCUMULATION_HISTORY
};

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


