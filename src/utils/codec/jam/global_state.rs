use sp_core::blake2_256;
use crate::types::{OpaqueHash, StateKey};
use crate::utils::codec::Encode;

pub trait StateKeyTrait {
    fn construct(&self) -> OpaqueHash;
}

impl StateKeyTrait for StateKey {
    fn construct(&self) -> OpaqueHash {
        let mut key_result = OpaqueHash::default();
        
        match self {
            StateKey::U8(value) => {
                key_result[..1].copy_from_slice(&value.encode());
            }
            StateKey::Service(value, service_id) => {
                key_result[..1].copy_from_slice(&value.encode());
                let service_encoded = u32::encode(service_id);
                key_result[1] = service_encoded[0];
                key_result[3] = service_encoded[1];
                key_result[5] = service_encoded[2];
                key_result[7] = service_encoded[3];
            }
            StateKey::Account(service_id, account) => {
                let service_encoded = u32::encode(service_id);
                let mut account_array = OpaqueHash::default();
                account_array[..account.len().min(32)].copy_from_slice(&account[..account.len().min(32)]);
                key_result[0] = service_encoded[0];
                key_result[1] = account_array[0];
                key_result[2] = service_encoded[1];
                key_result[3] = account_array[1];
                key_result[4] = service_encoded[2];
                key_result[5] = account_array[2];
                key_result[6] = service_encoded[3];
                key_result[7] = account_array[3];
                key_result[8..].copy_from_slice(&account_array[4..28]);
            }
        }

        key_result
    }
}

pub fn construct_storage_key(key: &OpaqueHash) -> OpaqueHash {
    let mut key_result = OpaqueHash::default();
    key_result[..4].copy_from_slice(&u32::MAX.encode());
    key_result[4..].copy_from_slice(&key[..28]);
    key_result
}

pub fn construct_preimage_key(hash: &OpaqueHash) -> OpaqueHash {
    let mut key_result = OpaqueHash::default();
    key_result[..4].copy_from_slice(&(u32::MAX - 1).encode());
    key_result[4..].copy_from_slice(&hash[1..29]);
    key_result
}

pub fn construct_lookup_key(hash: &OpaqueHash, length: u32) -> OpaqueHash {
    let mut key_result = OpaqueHash::default();
    key_result[..4].copy_from_slice(&length.encode());
    key_result[4..].copy_from_slice(&(blake2_256(hash)[2..30]));
    key_result
}

#[cfg(test)]
mod test {

    use crate::utils::codec::jam::global_state::StateKey;
    use super::*;

    #[test]
    fn test_state_key() {
        
        let result= StateKey::U8(5).construct();
        //let result = key.construct();
        println!("result = {:?} ", result);
        assert_eq!(1, 1);

        let key = StateKey::Service(2, 7);
        println!("Service = {:?}", key.construct());

        let key = StateKey::Account(4, vec![0, 1, 2]);
        println!("Account = {:?}", key.construct());

        let key = StateKey::Account(0xAABBCCDD, (0..=26).collect());
        println!("Account = {:?}", key.construct());

        let key = StateKey::Account(0xAABBCCDD, (0..=55).collect());
        println!("Account = {:?}", key.construct());
    }
}

/*


fn construct_state_key(key: StateKey) -> OpaqueHash {

    match key {
        StateKey::U64(_) => println!("u64 type"),
        StateKey::Service(_,_) => println!("(u64, ServiceId)"),
        StateKey::Account(_,_) => println!("(ServiceId, Vec<u8>"),
        _ => println!("Type not supported")
    }

    return OpaqueHash::default();
}*/


/*impl Encode for GlobalState {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
     
        self.auth_pools.encode_to(&mut blob);
        self.auth_queues.encode_to(&mut blob);
        self.recent_history.encode_to(&mut blob);
        self.safrole.encode_to(&mut blob);
        self.disputes.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.next_validators.encode_to(&mut blob);
        self.curr_validators.encode_to(&mut blob);
        self.prev_validators.encode_to(&mut blob);
        self.availability.encode_to(&mut blob); 
        self.time.encode_to(&mut blob);
        self.privileges.encode_to(&mut blob);
        self.statistics.encode_to(&mut blob);
        self.accumulation_history.encode_to(&mut blob);
        self.ready_queue.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for GlobalState {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(GlobalState {
            auth_pools: AuthPools::decode(blob)?,
            auth_queues: AuthQueues::decode(blob)?,
            recent_history: BlockHistory::decode(blob)?,
            safrole: Safrole::decode(blob)?,
            disputes: DisputesRecords::decode(blob)?,
            entropy: EntropyPool::decode(blob)?,
            next_validators: ValidatorsData::decode(blob)?,
            curr_validators: ValidatorsData::decode(blob)?,
            prev_validators: ValidatorsData::decode(blob)?,
            availability: AvailabilityAssignments::decode(blob)?,
            time: TimeSlot::decode(blob)?,
            privileges: Privileges::decode(blob)?,
            statistics: Statistics::decode(blob)?,
            accumulation_history: AccumulatedHistory::decode(blob)?,
            ready_queue: ReadyQueue::decode(blob)?,
            service_accounts: ServiceAccounts::decode(blob)?,
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct ServiceAccounts {
    pub service_accounts: Vec<AccountsMapEntry>,
}

impl Encode for ServiceAccounts {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        encode_unsigned(self.service_accounts.len() as usize).encode_to(&mut blob);
        for entry in &self.service_accounts {
            entry.encode_to(&mut blob);
        }
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServiceAccounts {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(ServiceAccounts { 
            service_accounts: { 
                let len = decode_unsigned(blob)? as usize;
                let mut service_accounts = Vec::with_capacity(len);
                for _ in 0..len {
                    service_accounts.push(AccountsMapEntry::decode(blob)?);
                }
                service_accounts
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountsMapEntry {
    pub id: ServiceId,
    pub data: AccountTest,
}

impl Encode for AccountsMapEntry {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.id.encode_to(&mut blob);
        self.data.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountsMapEntry {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountsMapEntry { 
            id: ServiceId::decode(blob)?,
            data: AccountTest::decode(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub service: ServiceInfo,
    pub preimages: Vec<PreimagesMapEntry>,
    pub lookup_meta: Vec<LookupMetaMapEntry>,
}

impl Encode for AccountTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.service.encode_to(&mut blob);
        encode_unsigned(self.preimages.len()).encode_to(&mut blob);
        for preimage in self.preimages.iter() {
            preimage.encode_to(&mut blob);
        }
        encode_unsigned(self.lookup_meta.len() as usize).encode_to(&mut blob);
        for entry in &self.lookup_meta {
            entry.encode_to(&mut blob);
        }
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountTest {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountTest { 
            service: ServiceInfo::decode(blob)?,
            preimages: (0..decode_unsigned(blob)?).map(|_| PreimagesMapEntry::decode(blob)).collect::<Result<Vec<_>, _>>()?,
            lookup_meta: { 
                let len = decode_unsigned(blob)? as usize;
                let mut lookup_meta = Vec::with_capacity(len);
                for _ in 0..len {
                    lookup_meta.push(LookupMetaMapEntry::decode(blob)?);
                }
                lookup_meta
            },
        })
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct LookupMetaMapEntry {
    pub key: LookupMetaMapKeyTest,
    pub value: Vec<TimeSlot>,
}

impl Encode for LookupMetaMapEntry {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.key.encode_to(&mut blob);
        encode_unsigned(self.value.len() as usize).encode_to(&mut blob);
        self.value.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for LookupMetaMapEntry {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LookupMetaMapEntry { 
            key: LookupMetaMapKeyTest::decode(reader)?,
            value: Vec::<TimeSlot>::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LookupMetaMapKeyTest {
    pub hash: HeaderHash,
    pub length: u32,
}

impl Encode for LookupMetaMapKeyTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.hash.encode_to(&mut blob);
        self.length.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for LookupMetaMapKeyTest {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LookupMetaMapKeyTest { 
            hash: HeaderHash::decode(reader)?,
            length: u32::decode(reader)?,
        })
    }
}*/