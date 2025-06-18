use vinwolf::types::{HeaderHash, OpaqueHash, ServicesStatisticsMapEntry, Preimage, PreimagesExtrinsic,Statistics, PreimagesMapEntry, ServiceId, TimeSlot};
use vinwolf::utils::codec::{BytesReader, Decode, Encode, EncodeLen, DecodeLen, ReadError};

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
    pub statistics: Vec<ServicesStatisticsMapEntry>,
}
impl Default for PreimagesState {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            statistics: Vec::new(),
        }
    }
}

impl Encode for PreimagesState {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.accounts.encode_len().encode_to(&mut blob);
        self.statistics.encode_len().encode_to(&mut blob);
        
        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for PreimagesState {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(PreimagesState { 
            accounts: Vec::<AccountsMapEntry>::decode_len(blob)?,
            statistics: Vec::<ServicesStatisticsMapEntry>::decode_len(blob)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountsMapEntry {
    pub id: ServiceId,
    pub data: AccountTest,
}
impl Default for AccountsMapEntry {
    fn default() -> Self {
        Self {
            id: ServiceId::default(),
            data: AccountTest { preimages: Vec::new(), lookup_meta: Vec::new() }
        }
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
            data: AccountTest::decode(blob)?,
        })
    }
}



#[derive(Debug, Clone, PartialEq)]
pub struct AccountTest {
    pub preimages: Vec<PreimagesMapEntry>,
    pub lookup_meta: Vec<LookupMetaMapEntry>,
}

impl Default for AccountTest {
    fn default() -> Self {
        Self {
            preimages: Vec::new(),
            lookup_meta: Vec::new(),
        }
    }
}

impl Encode for AccountTest {
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::new();

        self.preimages.encode_len().encode_to(&mut blob);
        self.lookup_meta.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccountTest {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(AccountTest { 
            preimages: Vec::<PreimagesMapEntry>::decode_len(blob)?,
            lookup_meta: Vec::<LookupMetaMapEntry>::decode_len(blob)?,
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
impl Default for LookupMetaMapEntry {
    fn default() -> Self {
        Self { key: LookupMetaMapKeyTest { hash: OpaqueHash::default(), length: u32::default() }, value: Vec::new() }    }
}

impl Encode for LookupMetaMapEntry {
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

impl Decode for LookupMetaMapEntry {
    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(LookupMetaMapEntry { 
            key: LookupMetaMapKeyTest::decode(reader)?,
            value: Vec::<TimeSlot>::decode(reader)?,
        })
    }
}





