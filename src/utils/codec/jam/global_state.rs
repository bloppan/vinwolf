use crate::jam_types::{KeyValue, StorageKey, RawState, StateRoot, *};
use crate::utils::codec::{Encode, EncodeLen, Decode, DecodeLen, ReadError, BytesReader};
use crate::utils::codec::generic::*;
use crate::constants::{*};

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


pub fn parse_state_keyvals(keyvals: &[KeyValue], state: &mut GlobalState) -> Result<(), ReadError> {

        for keyval in keyvals.iter() {
            
            let key = keyval.key;

            if is_simple_key(keyval) {

                let state_key = key[0] & 0xFF;
                let mut reader = BytesReader::new(&keyval.value);

                match state_key {
                    AUTH_POOLS => {
                        state.auth_pools = AuthPools::decode(&mut reader)?;
                    },
                    AUTH_QUEUE => {
                        state.auth_queues = AuthQueues::decode(&mut reader)?;
                    },
                    RECENT_HISTORY => {
                        state.recent_history = BlockHistory::decode(&mut reader)?;
                    },
                    SAFROLE => {
                        state.safrole = Safrole::decode(&mut reader)?;
                    },
                    DISPUTES => {
                        state.disputes = DisputesRecords::decode(&mut reader)?;
                    },
                    ENTROPY => {
                        state.entropy = EntropyPool::decode(&mut reader)?;
                    },
                    NEXT_VALIDATORS => {
                        state.next_validators = ValidatorsData::decode(&mut reader)?;
                    },
                    CURR_VALIDATORS => {
                        state.curr_validators = ValidatorsData::decode(&mut reader)?;
                    },
                    PREV_VALIDATORS => {
                        state.prev_validators = ValidatorsData::decode(&mut reader)?;
                    },
                    AVAILABILITY => {
                        state.availability = AvailabilityAssignments::decode(&mut reader)?;
                    },
                    TIME => {
                        state.time = TimeSlot::decode(&mut reader)?;
                    },
                    PRIVILEGES => {
                        state.privileges = Privileges::decode(&mut reader)?;
                    },
                    STATISTICS => {
                        state.statistics = Statistics::decode(&mut reader)?;
                    },
                    READY_QUEUE => {
                        state.ready_queue = ReadyQueue::decode(&mut reader)?;
                    },
                    ACCUMULATION_HISTORY => {
                        state.accumulation_history = AccumulatedHistory::decode(&mut reader)?;
                    },
                    _ => {
                        log::error!("Unknown key: {:?}", state_key);
                        return Err(ReadError::InvalidData);
                    },
                }

            } else if is_service_info_key(keyval) {

                let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");

                if state.service_accounts.get(&service_id).is_none() {
                    let account = Account::default();
                    state.service_accounts.insert(service_id, account);
                }
                let mut account_reader = BytesReader::new(&keyval.value);
                let mut account = state.service_accounts.get(&service_id).unwrap().clone();
                account.code_hash = OpaqueHash::decode(&mut account_reader).expect("Error decoding code_hash");
                account.balance = Gas::decode(&mut account_reader).expect("Error decoding balance") as u64;
                account.acc_min_gas = Gas::decode(&mut account_reader).expect("Error decoding acc_min_gas");
                account.xfer_min_gas = Gas::decode(&mut account_reader).expect("Error decoding xfer_min_gas");

                state.service_accounts.insert(service_id, account);
            } else {
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                let mut key_hash = vec![keyval.key[1], keyval.key[3], keyval.key[5]];
                key_hash.extend_from_slice(&keyval.key[7..]);

                // Preimage
                if is_preimage_key(keyval) { 
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    state.service_accounts.get_mut(&service_id).unwrap().preimages.insert(keyval.key, keyval.value.clone());

                // Storage
                } else if is_storage_key(keyval) {
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }

                    let mut storage_key  = [0u8; 31];
                    storage_key.copy_from_slice(&keyval.key);
                    state.service_accounts.get_mut(&service_id).unwrap().storage.insert(storage_key, keyval.value.clone());
                // Lookup
                } else {
                    let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                    let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                    
                    let mut timeslots_reader = BytesReader::new(&keyval.value);
                    let timeslots = Vec::<u32>::decode_len(&mut timeslots_reader).expect("Error decoding timeslots");
                    
                    if state.service_accounts.get(&service_id).is_none() {
                        state.service_accounts.insert(service_id, Account::default());
                    }
                    
                    let account = state.service_accounts.get_mut(&service_id).unwrap();
                    account.lookup.insert(keyval.key, timeslots.clone());
                }
            }
        }
        Ok(())
}

fn is_simple_key(keyval: &KeyValue) -> bool {

    keyval.key[0] <= 0x0F && keyval.key[1..].iter().all(|&b| b == 0)
}

fn is_service_info_key(keyval: &KeyValue) -> bool {

    keyval.key[0] == 0xFF && keyval.key[1..].iter().all(|&b| b == 0)
}

fn is_storage_key(keyval: &KeyValue) -> bool {

    keyval.key[1] == 0xFF && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
}

fn is_preimage_key(keyval: &KeyValue) -> bool {

    keyval.key[1] == 0xFE && keyval.key[3] == 0xFF && keyval.key[5] == 0xFF && keyval.key[7] == 0xFF
}

/*fn is_lookup_key(keyval: &KeyValue) -> bool {
    
    !is_simple_key(keyval) && !is_service_info_key(keyval) && !is_storage_key(keyval) && !is_preimage_key(keyval)
}*/
