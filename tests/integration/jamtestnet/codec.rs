use vinwolf::types::{
    AccumulatedHistory, AuthPools, AuthQueues, AvailabilityAssignments, BlockHistory, DisputesRecords, EntropyPool, 
    HeaderHash, PreimagesMapEntry, Privileges, ReadyQueue, Safrole, ServiceId, ServiceInfo, Statistics, TimeSlot, 
    ValidatorsData, 

};
use vinwolf::utils::codec::generic::{decode_unsigned, encode_unsigned};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalStateTest {
    pub time: TimeSlot,
    pub availability: AvailabilityAssignments,
    pub entropy: EntropyPool,
    pub recent_history: BlockHistory,
    pub auth_pools: AuthPools,
    pub auth_queues: AuthQueues,
    pub statistics: Statistics,
    pub prev_validators: ValidatorsData,
    pub curr_validators: ValidatorsData,
    pub next_validators: ValidatorsData,
    pub disputes: DisputesRecords,
    pub safrole: Safrole,
    pub service_accounts: ServiceAccounts,
    pub accumulation_history: AccumulatedHistory,
    pub ready_queue: ReadyQueue,
    pub privileges: Privileges,
}

impl Encode for GlobalStateTest {
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
        self.service_accounts.encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for GlobalStateTest {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(GlobalStateTest {
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
        let id = ServiceId::decode(blob)?;
        let data = AccountTest::decode(blob)?;
        println!("data: {:x?}", data);
        return Ok(AccountsMapEntry { 
            id,
            data,
        })
        /*Ok(AccountsMapEntry { 
            id: ServiceId::decode(blob)?,
            data: AccountTest::decode(blob)?,
        })*/
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
}