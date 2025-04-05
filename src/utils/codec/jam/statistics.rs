use hex::decode;

use crate::types::{
    ActivityRecord, ActivityRecords, CoreActivityRecord, Statistics, CoresStatistics, SeviceActivityRecord, ServicesStatisticsMapEntry,
    ServicesStatistics, ServiceId
};
use crate::utils::codec::{BytesReader, Decode, Encode, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

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

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

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
        
        let mut records= ActivityRecords::default();

        for record in records.records.iter_mut() {
            *record = ActivityRecord::decode(blob)?;
        }

        return Ok(records);
    }
}

impl Encode for CoreActivityRecord {
    
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        encode_unsigned(self.gas_used as usize).encode_to(&mut blob);
        encode_unsigned(self.imports as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_count as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_size as usize).encode_to(&mut blob);
        encode_unsigned(self.exports as usize).encode_to(&mut blob);
        encode_unsigned(self.bundle_size as usize).encode_to(&mut blob);
        encode_unsigned(self.da_load as usize).encode_to(&mut blob);
        encode_unsigned(self.popularity as usize).encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for CoreActivityRecord {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(CoreActivityRecord {
            gas_used: decode_unsigned(blob)? as u64,
            imports: decode_unsigned(blob)? as u16,
            extrinsic_count: decode_unsigned(blob)? as u16,
            extrinsic_size: decode_unsigned(blob)? as u32,
            exports: decode_unsigned(blob)? as u16,
            bundle_size: decode_unsigned(blob)? as u32,
            da_load: decode_unsigned(blob)? as u32,
            popularity: decode_unsigned(blob)? as u16,
        })
    }
}

impl Encode for CoresStatistics {
    
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        for record in self.records.iter() {
            record.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for CoresStatistics {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let mut cores = CoresStatistics::default();

        for record in cores.records.iter_mut() {
            *record = CoreActivityRecord::decode(blob)?;
        }
        return Ok(cores);
    }
}

impl Encode for SeviceActivityRecord {
    
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.provided_count.encode_to(&mut blob);
        self.provided_size.encode_to(&mut blob);
        self.refinement_count.encode_to(&mut blob);
        self.refinement_gas_used.encode_to(&mut blob);
        self.imports.encode_to(&mut blob);
        self.extrinsic_count.encode_to(&mut blob);
        self.extrinsic_size.encode_to(&mut blob);
        self.exports.encode_to(&mut blob);
        self.accumulate_count.encode_to(&mut blob);
        self.accumulate_gas_used.encode_to(&mut blob);
        self.on_transfers_count.encode_to(&mut blob);
        self.on_transfers_gas_used.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for SeviceActivityRecord {

    fn decode(reader: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(SeviceActivityRecord {
                provided_count: u16::decode(reader)?,
                provided_size: u32::decode(reader)?,
                refinement_count: u32::decode(reader)?,
                refinement_gas_used: u64::decode(reader)?,
                imports: u32::decode(reader)?,
                extrinsic_count: u32::decode(reader)?,
                extrinsic_size: u32::decode(reader)?,
                exports: u32::decode(reader)?,
                accumulate_count: u32::decode(reader)?,
                accumulate_gas_used: u64::decode(reader)?,
                on_transfers_count: u32::decode(reader)?,
                on_transfers_gas_used: u64::decode(reader)?,
            }
        )
    }
}

impl Encode for ServicesStatisticsMapEntry {
    
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.id.encode_to(&mut blob);
        self.record.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServicesStatisticsMapEntry {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(ServicesStatisticsMapEntry {
            id: ServiceId::decode(blob)?,
            record: SeviceActivityRecord::decode(blob)?,
        })
    }
}

impl Encode for ServicesStatistics {
    
    fn encode(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        encode_unsigned(self.records.len()).encode_to(&mut blob);

        for (id, record) in self.records.iter() {
            id.encode_to(&mut blob);
            record.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ServicesStatistics {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let len = decode_unsigned(blob)?;
        let mut map = ServicesStatistics::default();

        for _ in 0..len {
            let id = ServiceId::decode(blob)?;
            let record = SeviceActivityRecord::decode(blob)?;
            map.records.insert(id, record);
        }

        return Ok(map);
    }
}

impl Encode for Statistics {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::with_capacity(std::mem::size_of::<Self>());

        self.curr.encode_to(&mut blob);
        self.prev.encode_to(&mut blob);
        self.cores.encode_to(&mut blob);
        self.services.encode_to(&mut blob);

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
            cores: CoresStatistics::decode(blob)?,
            services: ServicesStatistics::decode(blob)?,
        })
    }
}

