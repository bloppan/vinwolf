/*
    The serialization of state primarily involves placing all the various components of σ into a single
    mapping from 32-octet sequence state-keys to octet sequences of indefinite length. The state-key is constructed from a
    hash component and a chapter component, equivalent to either the index of a state component or, in the case of the
    inner dictionaries of δ, a service index.
*/

use sp_core::blake2_256;
use jam_types::{OpaqueHash, StateKeyType, StateKey, SerializedState, GlobalState, ServiceInfo, StorageKey};
use codec::{Encode, EncodeLen};
use crate::{common::get_footprint_and_threshold};
use constants::node::{
    ACCUMULATION_HISTORY, AUTH_POOLS, AUTH_QUEUE, AVAILABILITY, CURR_VALIDATORS, DISPUTES, ENTROPY, NEXT_VALIDATORS, PREV_VALIDATORS, PRIVILEGES, 
    READY_QUEUE, RECENT_HISTORY, SAFROLE, STATISTICS, TIME
};

// The state serialization is then defined as the dictionary built from the amalgamation of each of the components.
// Cryptographic hashing ensures that there will be no duplicate state-keys given that there are no duplicate inputs to C.
// Formally, we define serialize which transforms some state σ into its serialized form:
pub fn serialize(global_state: &GlobalState) -> SerializedState {

    let mut state = SerializedState::default();

    state.map.insert(StateKeyType::U8(AUTH_POOLS).construct(), global_state.auth_pools.encode());
    state.map.insert(StateKeyType::U8(AUTH_QUEUE).construct(), global_state.auth_queues.encode());
    state.map.insert(StateKeyType::U8(RECENT_HISTORY).construct(), global_state.recent_history.encode());
    state.map.insert(StateKeyType::U8(SAFROLE).construct(), global_state.safrole.encode());
    state.map.insert(StateKeyType::U8(DISPUTES).construct(), global_state.disputes.encode());
    state.map.insert(StateKeyType::U8(ENTROPY).construct(), global_state.entropy.encode());
    state.map.insert(StateKeyType::U8(NEXT_VALIDATORS).construct(), global_state.next_validators.encode());
    state.map.insert(StateKeyType::U8(CURR_VALIDATORS).construct(), global_state.curr_validators.encode());
    state.map.insert(StateKeyType::U8(PREV_VALIDATORS).construct(), global_state.prev_validators.encode());
    state.map.insert(StateKeyType::U8(AVAILABILITY).construct(), global_state.availability.encode());
    state.map.insert(StateKeyType::U8(TIME).construct(), global_state.time.encode());
    state.map.insert(StateKeyType::U8(PRIVILEGES).construct(), global_state.privileges.encode());
    state.map.insert(StateKeyType::U8(STATISTICS).construct(), global_state.statistics.encode());
    state.map.insert(StateKeyType::U8(READY_QUEUE).construct(), global_state.ready_queue.encode());
    state.map.insert(StateKeyType::U8(ACCUMULATION_HISTORY).construct(), global_state.accumulation_history.encode());
    
    for (service_id, account) in global_state.service_accounts.iter() {
    
        let key = StateKeyType::Service(255, *service_id).construct();
        let (items, octets, _threshold) = get_footprint_and_threshold(account);

        let service_info = ServiceInfo {
            balance: account.balance,
            code_hash: account.code_hash,
            acc_min_gas: account.acc_min_gas,
            xfer_min_gas: account.xfer_min_gas,
            bytes: octets, 
            items: items,
        };

        state.map.insert(key, service_info.encode());

        for preimage in account.preimages.iter() {
            state.map.insert(*preimage.0, preimage.1.encode());
        }
        
        for lookup in account.lookup.iter() {
            state.map.insert(*lookup.0, lookup.1.encode_len());
        }

        for item in account.storage.iter() {
            state.map.insert(*item.0, item.1.encode());
        }
    }
    
    return state;
}


pub trait StateKeyTrait {
    fn construct(&self) -> StateKey;
}

impl StateKeyTrait for StateKeyType {

    // We define the state-key constructor functions C as:
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
    let mut key_result = StateKey::default();
    key_result[..4].copy_from_slice(&u32::MAX.encode());
    key_result[4..].copy_from_slice(&key[..27]);
    key_result
}

pub fn construct_preimage_key(hash: &OpaqueHash) -> StateKey {
    let mut key_result = StateKey::default();
    key_result[..4].copy_from_slice(&(u32::MAX - 1).encode());
    key_result[4..].copy_from_slice(&hash[1..28]);
    key_result
}

pub fn construct_lookup_key(hash: &OpaqueHash, length: u32) -> StateKey {
    let mut key_result = StateKey::default();
    key_result[..4].copy_from_slice(&length.encode());
    key_result[4..].copy_from_slice(&(blake2_256(hash)[2..29]));
    key_result
}

#[cfg(test)]
mod test {

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
