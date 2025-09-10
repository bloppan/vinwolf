use std::collections::{HashSet, HashMap};
use std::hash::Hash;
use super::{log, hex};
use sp_core::{ed25519, Pair};
use std::path::Path;
use std::fs::File;
use std::io::Read;

use jam_types::{
    Account, AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, Balance, BandersnatchPublic, BlsPublic, DisputesRecords, Ed25519Public, 
    Ed25519Signature, EntropyPool, Gas, GlobalState, KeyValue, Metadata, OpaqueHash, PreimageData, Privileges, ReadError, ReadyQueue, RecentAccOutputs, 
    RecentBlocks, Safrole, ServiceAccounts, ServiceId, StateKeyType, Statistics, TimeSlot, ValidatorsData, ServiceInfo
};
use constants::node::{
    MIN_BALANCE, MIN_BALANCE_PER_ITEM, MIN_BALANCE_PER_OCTET, AUTH_POOLS, AUTH_QUEUE, RECENT_HISTORY, SAFROLE, DISPUTES, ENTROPY, NEXT_VALIDATORS, 
    CURR_VALIDATORS, PREV_VALIDATORS, AVAILABILITY, TIME, PRIVILEGES, STATISTICS, READY_QUEUE, ACCUMULATION_HISTORY, RECENT_ACC_OUTPUTS
};
use crate::serialization::{StateKeyTrait, construct_lookup_key, construct_preimage_key};
use codec::{ Encode, Decode, DecodeLen, BytesReader};
use codec::generic_codec::{decode_unsigned, decode};

pub fn dict_subtract<K: Eq + std::hash::Hash + Clone, V: Clone>(
    d: &HashMap<K, V>,
    s: &HashSet<K>,
) -> HashMap<K, V> {
    d.iter()
        .filter(|(k, _)| !s.contains(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/*pub fn get_footprint_and_threshold(account: &Account) -> (u32, u64, Balance) {

    let items: u32 = 2 * account.lookup.len() as u32 + account.storage.len() as u32;

    let mut octets: u64 = 0;

    for (lookup_key, _timeslot) in account.lookup.iter() {
        let length = u32::from_le_bytes([lookup_key[1], lookup_key[3], lookup_key[5], lookup_key[7]]);
        octets += 81 + length as u64;
    }
    for (_key, value) in account.storage.iter() {
        octets += 34 + value.0 as u64 + value.1.len() as u64;
    }

    let threshold: Balance = std::cmp::max(0, MIN_BALANCE + items as Balance * MIN_BALANCE_PER_ITEM + octets as Balance * MIN_BALANCE_PER_OCTET - account.gratis_storage_offset);
    //log::debug!("Items: {:?}, octets: {:?}, threshold: {:?}", items, octets, threshold);
    return (items, octets, threshold);
}*/   

pub fn get_threshold(account: &Account) -> Balance {
    std::cmp::max(0, (MIN_BALANCE + account.items as Balance * MIN_BALANCE_PER_ITEM + account.octets as Balance * MIN_BALANCE_PER_OCTET).saturating_sub(account.gratis_storage_offset)) as Balance
}

pub fn update_storage_octets(init_value: &[u8], final_value: &[u8]) -> i64 {
    (final_value.len() - init_value.len()) as i64
}

pub fn octets_lookup(length: u32) -> u64 {
    (81 + length) as u64
}

pub fn keys_to_set<K: Eq + std::hash::Hash + Clone, V>(
    map: &HashMap<K, V>,
) -> HashSet<K> {
    map.keys().cloned().collect()
}

pub fn is_sorted_and_unique<T: PartialOrd + Hash + Eq>(vec: &[T]) -> bool {
    let mut seen = HashSet::new();

    if vec.len() < 2 {
        return true;
    }
    
    vec.windows(2).all(|window| window[0] < window[1]) && vec.iter().all(|x| seen.insert(x))
}

pub fn has_duplicates<T: Eq + std::hash::Hash + Clone>(items: &[T]) -> bool {
    let mut seen = HashSet::<T>::new();
    for i in items {
        if !seen.insert(i.clone()) {
            return true;
        }
    }
    false
}

pub fn bad_order<T: PartialOrd>(items: &[T]) -> bool {

    if items.len() < 2 {
        return false;
    }

    for i in 0..items.len() - 1 {
        if items[i] > items[i + 1] {
            return true; // Bad order 
        }
    }
    
    return false; // Order correct
}

pub trait VerifySignature {
    fn verify_signature(&self, message: &[u8], public_key: &Ed25519Public) -> bool;
}

impl VerifySignature for Ed25519Signature {
    
    fn verify_signature(&self, message: &[u8], public_key: &Ed25519Public) -> bool {

        let signature = ed25519::Signature::from_raw(*self);
        let public_key = ed25519::Public::from_raw(*public_key);

        ed25519::Pair::verify(&signature, message, &public_key)
    }
}

pub fn set_offenders_null(validators_data: &mut ValidatorsData, offenders: &[Ed25519Public]) {
    
    // We return the same keyset if there aren't offenders
    if offenders.is_empty() {
        return;
    }

    // For each offender set ValidatorData to zero
    'next_offender: for offender in offenders {
        for validator in validators_data.list.iter_mut() {
            if *offender == validator.ed25519 {
                log::debug!("Validator {} belongs to offenders set", crate::print_hash!(validator.ed25519));
                validator.bandersnatch = [0u8; std::mem::size_of::<BandersnatchPublic>()];
                validator.ed25519 = [0u8; std::mem::size_of::<Ed25519Public>()];
                validator.bls = [0u8; std::mem::size_of::<BlsPublic>()];
                validator.metadata = [0u8; std::mem::size_of::<Metadata>()];
                continue 'next_offender;
            }
        }
    }
}

pub fn parse_preimage(service_accounts: &ServiceAccounts, service_id: &ServiceId) -> Result<Option<PreimageData>, ReadError> {

    let preimage_blob = if let Some(account) = service_accounts.get(service_id) {
        let preimage_key = StateKeyType::Account(*service_id, construct_preimage_key(&account.code_hash)).construct();
        if let Some(preimage) = account.storage.get(&preimage_key) {
            preimage
        } else {
            log::error!("Preimage key {} not found for service: {:?}. Code hash: {}", hex::encode(&preimage_key), service_id, hex::encode(account.code_hash));
            return Ok(None);
        }
    } else {
        log::error!("Account not found for service: {:?}", service_id);
        return Ok(None);
    };

    let preimage = decode_preimage(&preimage_blob)?;

    return Ok(Some(preimage));
}

pub fn decode_preimage(preimage_blob: &[u8]) -> Result<PreimageData, ReadError> {
    
    let mut preimage_reader = BytesReader::new(preimage_blob);
    let metadata_len = decode_unsigned(&mut preimage_reader)?;
    let metadata = preimage_reader.read_bytes(metadata_len as usize)?.to_vec();
    let preimage_len = preimage_reader.data.len(); 
    let code = preimage_reader.read_bytes(preimage_len - metadata_len as usize - 1)?.to_vec();

    Ok(PreimageData {
        metadata,
        code,
    })
}

pub fn historical_preimage_lookup(
            service_id: &ServiceId, 
            account: &Account, 
            slot: &TimeSlot, 
            hash: &OpaqueHash
    ) -> Option<Vec<u8>> {

    let preimage_key = StateKeyType::Account(*service_id, construct_preimage_key(hash).to_vec()).construct();
    
    if let Some(preimage) = account.storage.get(&preimage_key) {
        let length = preimage.len() as u32;
        let lookup_key = StateKeyType::Account(*service_id, construct_lookup_key(hash, length).to_vec()).construct();
        if let Some(timeslot_record) = account.storage.get(&lookup_key) {
            let mut reader = BytesReader::new(timeslot_record);
            let timeslots = Vec::<TimeSlot>::decode_len(&mut reader).unwrap(); // TODO handle error
            if check_preimage_availability(&timeslots, slot) {
                return Some(preimage.clone());
            }
        }
    }

    return None;
}

fn check_preimage_availability(timeslot_record: &[TimeSlot], slot: &TimeSlot) -> bool {
 
    if timeslot_record.len() == 0 {
        return false;
    } else if timeslot_record.len() == 1 {
        if timeslot_record[0] <= *slot {
            return true;
        }
    } else if timeslot_record.len() == 2 {
        if timeslot_record[0] <= *slot && *slot < timeslot_record[1] {
            return true;
        }
    } else if timeslot_record.len() == 3 {
        if (timeslot_record[0] <= *slot && *slot < timeslot_record[1]) || (timeslot_record[2] <= *slot) {
            return true;
        }
    }
 
    return false;
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
                        state.recent_history = RecentBlocks::decode(&mut reader)?;
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
                    RECENT_ACC_OUTPUTS => {
                        state.recent_acc_outputs = RecentAccOutputs::decode(&mut reader)?;
                    }
                    _ => {
                        log::error!("Unknown key: {:?}", state_key);
                        return Err(ReadError::InvalidData);
                    },
                }
            } else if is_service_info_key(keyval) {
                /*let mut service_reader = BytesReader::new(&keyval.key[1..]);
                let service_id = ServiceId::decode(&mut service_reader).expect("Error decoding service id");*/
                let service_id_vec = vec![keyval.key[1], keyval.key[3], keyval.key[5], keyval.key[7]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());

                //log::info!("Service: {:?} info key: {}", service_id, hex::encode(&keyval.key));

                let mut account_reader = BytesReader::new(&keyval.value);
                let service_info = ServiceInfo::decode(&mut account_reader).expect("Error decoding service info");

                if state.service_accounts.get(&service_id).is_none() {
                    state.service_accounts.insert(service_id, Account::default());
                }

                state.service_accounts.get_mut(&service_id).unwrap().code_hash = service_info.code_hash;
                state.service_accounts.get_mut(&service_id).unwrap().balance = service_info.balance;
                state.service_accounts.get_mut(&service_id).unwrap().acc_min_gas = service_info.acc_min_gas;
                state.service_accounts.get_mut(&service_id).unwrap().xfer_min_gas = service_info.xfer_min_gas;
                state.service_accounts.get_mut(&service_id).unwrap().created_at = service_info.created_at;
                state.service_accounts.get_mut(&service_id).unwrap().last_acc = service_info.last_acc;
                state.service_accounts.get_mut(&service_id).unwrap().parent_service = service_info.parent_service;
                state.service_accounts.get_mut(&service_id).unwrap().octets = service_info.octets;
                state.service_accounts.get_mut(&service_id).unwrap().items = service_info.items;
                state.service_accounts.get_mut(&service_id).unwrap().gratis_storage_offset = service_info.gratis_storage_offset;

            } else {
                
                let service_id_vec = vec![keyval.key[0], keyval.key[2], keyval.key[4], keyval.key[6]];
                let service_id = decode::<ServiceId>(&service_id_vec, std::mem::size_of::<ServiceId>());
                
                //log::info!("Service: {:?} Account key: {}", service_id, hex::encode(&keyval.key));
                if state.service_accounts.get(&service_id).is_none() {
                    state.service_accounts.insert(service_id, Account::default());
                }

                state.service_accounts.get_mut(&service_id).unwrap().storage.insert(keyval.key, keyval.value.clone());
            }
        }
        Ok(())
}

fn is_simple_key(keyval: &KeyValue) -> bool {

    keyval.key[0] <= 0x10 && keyval.key[1..].iter().all(|&b| b == 0)
}

fn is_service_info_key(keyval: &KeyValue) -> bool {

    keyval.key[0] == 0xFF && keyval.key[2] == 0x00 && keyval.key[4] == 0x00 && keyval.key[6] == 0x00 && keyval.key[8..].iter().all(|&b| b == 0)
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

pub fn read_bin_file(path: &Path) -> Result<Vec<u8>, ()> {
    
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => { log::error!("Failed to open file {}: {}", path.display(), e); return Err(()) },
    };

    let mut test_content = Vec::new();

    match file.read_to_end(&mut test_content) {
        Ok(_) => { return Ok(test_content) },
        Err(e) => { log::error!("Failed to read file {}: {}", path.display(), e); return Err(()) },
    }
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



/*#[cfg(test)]
mod tests {

    use super::*;
    use crate::jam_types::{PreimagesErrorCode, Preimage};

    #[test]
    fn test_preimages_extrinsic_process() {
        let mut services = ServiceAccounts::default();
        let slot = TimeSlot::default();

        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 2, blob: vec![7, 8, 9] },
        ];
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(&mut services, &slot), Err(ProcessError::PreimagesError(PreimagesErrorCode::RequesterNotFound)));

        let preimages = vec![
            Preimage { requester: 0, blob: vec![1, 2, 3] },
            Preimage { requester: 1, blob: vec![4, 5, 6] },
            Preimage { requester: 0, blob: vec![7, 8, 9] },
        ];
    
        let preimages_extrinsic = PreimagesExtrinsic { preimages };
        assert_eq!(preimages_extrinsic.process(&mut services, &slot), Err(ProcessError::PreimagesError(PreimagesErrorCode::PreimagesNotSortedOrUnique)));
    }

}*/

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn integer_sorted() {
        let integers = vec![1, 2, 4, 6, 8];
        assert_eq!(true, is_sorted_and_unique(&integers));
        let integers2 = vec![1, 2, 3, 3, 4, 6];
        assert_eq!(false, is_sorted_and_unique(&integers2));
    }

    #[test]
    fn array_sorted() {
        let array_1: [u8; 5] = [0, 1, 2, 3, 4];
        let array_2: [u8; 5] = [1, 2, 3, 4, 5];
        let array_3: [u8; 5] = [0, 1, 3, 3, 4];
        let array_4: [u8; 5] = [0, 1, 3, 3, 4];

        let vector: Vec<[u8; 5]> = vec![array_1, array_2];
        let vector2: Vec<[u8; 5]> = vec![array_2, array_1];
        let vector3: Vec<[u8; 5]> = vec![array_1, array_3];
        let vector4: Vec<[u8; 5]> = vec![array_3, array_4];
        let vector5: Vec<[u8; 5]> = vec![array_2, array_4];

        assert_eq!(true, is_sorted_and_unique(&vector));
        assert_eq!(false, is_sorted_and_unique(&vector2));
        assert_eq!(true, is_sorted_and_unique(&vector3));
        assert_eq!(false, is_sorted_and_unique(&vector4));
        assert_eq!(false, is_sorted_and_unique(&vector5));
    }

    #[test]
    fn dict_subtract_test() {
        let mut d = HashMap::new();
        d.insert("a", 1);
        d.insert("b", 2);
        d.insert("c", 3);
    
        let mut s = HashSet::new();
        s.insert("b");
        s.insert("c");
    
        let result = dict_subtract(&d, &s);
        println!("{:?}", result); // Must print {"a": 1}
    }

    #[test]
    fn keys_to_set_test() {
        let mut map = HashMap::new();
        map.insert("a", 10);
        map.insert("b", 20);
        map.insert("c", 30);
    
        let key_set: HashSet<_> = keys_to_set(&map);
    
        println!("{:?}", key_set); // {"a", "b", "c"}
    }

    #[test]
    fn hashmap_extend_test() {
        let mut map1 = HashMap::new();
        map1.insert("a", 1);
        map1.insert("b", 2);
    
        let mut map2 = HashMap::new();
        map2.insert("c", 3);
        map2.insert("d", 4);
        map2.insert("a", 5); // This will overwrite the value for "a" in map1
        map1.extend(map2);
    
        println!("{:?}", map1); // {"a": 5, "b": 2, "c": 3, "d": 4}
    }
}
