use crate::types::{Statistics, ActivityRecord, ActivityRecords};
use crate:: utils::codec::{BytesReader, Decode, Encode, ReadError};

impl Encode for ActivityRecord {

    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<ActivityRecord>());

        self.blocks.encode_to(&mut blob);
        self.tickets.encode_to(&mut blob);
        self.preimages.encode_to(&mut blob);
        self.preimages_size.encode_to(&mut blob);
        self.guarantees.encode_to(&mut blob);
        self.assurances.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ActivityRecord {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(ActivityRecord {
            blocks: u32::decode(blob)?,
            tickets: u32::decode(blob)?,
            preimages: u32::decode(blob)?,
            preimages_size: u32::decode(blob)?,
            guarantees: u32::decode(blob)?,
            assurances: u32::decode(blob)?,
        })
    }
}

impl Encode for ActivityRecords {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<ActivityRecords>());

        for record in self.records.iter() {
            record.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ActivityRecords {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let activity_record = ActivityRecord {
            blocks: 0,
            tickets: 0,
            preimages: 0,
            preimages_size: 0,
            guarantees: 0,
            assurances: 0,
        };

        let mut records: ActivityRecords = ActivityRecords { records: Box::new(core::array::from_fn(|_| activity_record.clone())) };

        for record in records.records.iter_mut() {
            *record = ActivityRecord::decode(blob)?;
        }

        return Ok(records);
    }
}

impl Encode for Statistics {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Statistics>());

        self.curr.encode_to(&mut blob);
        self.prev.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Statistics {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(Statistics {
            curr: ActivityRecords::decode(blob)?,
            prev: ActivityRecords::decode(blob)?,
        })
    }
}

