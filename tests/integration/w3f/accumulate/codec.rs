use vinwolf::types::{AccumulatedHistory, OpaqueHash, PreimagesMapEntry, Privileges, ReadyQueue, ServiceId, ServiceInfo, ServicesStatistics, TimeSlot, WorkReport};
use vinwolf::utils::codec::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct InputAccumulate {
    pub slot: TimeSlot,
    pub reports: Vec<WorkReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountsMapEntry {
    pub id: ServiceId,
    pub data: AccountTest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageMapEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub service: ServiceInfo,
    pub storage: Vec<StorageMapEntry>,
    pub preimages: Vec<PreimagesMapEntry>,
}

impl Default for StorageMapEntry {
    fn default() -> Self {
        Self { key: vec![], value: vec![] }
    }
}

impl Default for AccountTest {
    fn default() -> Self {
        Self {
            service: ServiceInfo::default(),
            storage: vec![],
            preimages: vec![],
        }
    }
}

impl Encode for AccountTest {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.service.encode_to(&mut blob);
        self.storage.encode_len().encode_to(&mut blob);
        self.preimages.encode_len().encode_to(&mut blob);

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
            storage: Vec::<StorageMapEntry>::decode_len(blob)?,
            preimages: Vec::<PreimagesMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for StorageMapEntry {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.key.encode_len().encode_to(&mut blob);
        self.value.encode_len().encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, writer: &mut Vec<u8>) {
        writer.extend_from_slice(&self.encode());
    }
}

impl Decode for StorageMapEntry {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(Self { key: Vec::<u8>::decode_len(reader)?, value: Vec::<u8>::decode_len(reader)? })
    }
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
            data: {
                let data= AccountTest::decode(blob)?;
                data
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateAccumulate {
    pub slot: TimeSlot,
    pub entropy: OpaqueHash,
    pub ready: ReadyQueue,
    pub accumulated: AccumulatedHistory,
    pub privileges: Privileges,
    pub statistics: ServicesStatistics,
    pub accounts: Vec<AccountsMapEntry>,
}
impl Default for AccountsMapEntry {
    fn default() -> Self {
        Self {
            id: 0,
            data: AccountTest::default(),
        }
    }
}
impl Encode for StateAccumulate {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.slot.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.ready.encode_to(&mut blob);
        self.accumulated.encode_to(&mut blob);
        self.privileges.encode_to(&mut blob);
        self.statistics.encode_to(&mut blob);
        self.accounts.encode_len().encode_to(&mut blob);
                
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for StateAccumulate {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(StateAccumulate { 
            slot: TimeSlot::decode(blob)?,
            entropy: OpaqueHash::decode(blob)?,
            ready: ReadyQueue::decode(blob)?,
            accumulated: AccumulatedHistory::decode(blob)?,
            privileges: Privileges::decode(blob)?,
            statistics: ServicesStatistics::decode(blob)?,
            accounts: Vec::<AccountsMapEntry>::decode_len(blob)?,
        })
    }
}

impl Encode for InputAccumulate {
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.slot.encode_to(&mut blob);
        self.reports.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputAccumulate {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputAccumulate { 
            slot: TimeSlot::decode(blob)?,
            reports: Vec::<WorkReport>::decode_len(blob)?,
        })
    }
}