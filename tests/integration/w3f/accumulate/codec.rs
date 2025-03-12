use vinwolf::types::{AccumulatedHistory, Entropy, PreimagesMapEntry, Privileges, ReadyQueue, ServiceId, ServiceInfo, TimeSlot, WorkReport};
use vinwolf::utils::codec::generic::{decode_unsigned, encode_unsigned};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub service: ServiceInfo,
    pub preimages: Vec<PreimagesMapEntry>,
}

impl Encode for AccountTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.service.encode_to(&mut blob);
        encode_unsigned(self.preimages.len()).encode_to(&mut blob);
        for preimage in self.preimages.iter() {
            preimage.encode_to(&mut blob);
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
pub struct StateAccumulate {
    pub slot: TimeSlot,
    pub entropy: Entropy,
    pub ready: ReadyQueue,
    pub accumulated: AccumulatedHistory,
    pub privileges: Privileges,
    pub accounts: Vec<AccountsMapEntry>,
}

impl Encode for StateAccumulate {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.slot.encode_to(&mut blob);
        self.entropy.encode_to(&mut blob);
        self.ready.encode_to(&mut blob);
        self.accumulated.encode_to(&mut blob);
        self.privileges.encode_to(&mut blob);
        encode_unsigned(self.accounts.len()).encode_to(&mut blob);
        for account in self.accounts.iter() {
            account.encode_to(&mut blob);
        }
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
            entropy: Entropy::decode(blob)?,
            ready: ReadyQueue::decode(blob)?,
            accumulated: AccumulatedHistory::decode(blob)?,
            privileges: Privileges::decode(blob)?,
            accounts: (0..decode_unsigned(blob)?).map(|_| AccountsMapEntry::decode(blob)).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputAccumulate {
    pub slot: TimeSlot,
    pub reports: Vec<WorkReport>,
}

impl Encode for InputAccumulate {
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.slot.encode_to(&mut blob);
        self.reports.encode_to(&mut blob);

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
            reports: Vec::<WorkReport>::decode(blob)?,
        })
    }
}