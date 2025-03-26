use vinwolf::types::{HeaderHash, TimeSlot, PreimagesExtrinsic, PreimagesMapEntry, ServiceId};
use vinwolf::utils::codec::generic::{decode_unsigned, encode_unsigned};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, ReadError};

#[derive(Debug, Clone, PartialEq)]
pub struct InputPreimages {
    pub preimages: PreimagesExtrinsic,
    pub slot: TimeSlot,
}

impl Encode for InputPreimages {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        self.preimages.encode_to(&mut blob);
        self.slot.encode_to(&mut blob);
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for InputPreimages {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(InputPreimages { 
            preimages: PreimagesExtrinsic::decode(reader)?,
            slot: TimeSlot::decode(reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreimagesState {
    pub accounts: Vec<AccountsMapEntry>,
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

impl Encode for PreimagesState {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        encode_unsigned(self.accounts.len() as usize).encode_to(&mut blob);
        for entry in &self.accounts {
            entry.encode_to(&mut blob);
        }
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PreimagesState {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(PreimagesState { 
            accounts: { 
                let len = decode_unsigned(blob)? as usize;
                let mut accounts = Vec::with_capacity(len);
                for _ in 0..len {
                    accounts.push(AccountsMapEntry::decode(blob)?);
                }
                accounts
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub preimages: Vec<PreimagesMapEntry>,
    pub lookup_meta: Vec<LookupMetaMapEntry>,
}

impl Encode for AccountTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();
        encode_unsigned(self.preimages.len() as usize).encode_to(&mut blob);
        for entry in &self.preimages {
            entry.encode_to(&mut blob);
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
            preimages: { 
                let len = decode_unsigned(blob)? as usize;
                let mut preimages = Vec::with_capacity(len);
                for _ in 0..len {
                    preimages.push(PreimagesMapEntry::decode(blob)?);
                }
                preimages
            },
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





