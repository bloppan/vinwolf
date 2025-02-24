use ark_ec_vrfs::suites::bandersnatch::edwards::Output;

use crate::constants::EPOCH_LENGTH;
use crate::types::{
    AccumulateRoot, AccumulatedHistory, AlwaysAccumulateMapItem, Gas, OutputAccumulation, ReadyQueue, ReadyRecord, ServiceId, WorkPackageHash, WorkReport
};
use crate::utils::codec::{Encode, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for ReadyRecord {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.report.encode_to(&mut blob);
        encode_unsigned(self.dependencies.len()).encode_to(&mut blob);
        for dep in self.dependencies.iter() {
            dep.encode_to(&mut blob);
        }

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReadyRecord {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(ReadyRecord {
            report: WorkReport::decode(blob)?,
            dependencies: (0..decode_unsigned(blob)?).map(|_| WorkPackageHash::decode(blob)).collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Encode for ReadyQueue {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        for item in self.queue.iter() {
            encode_unsigned(item.len()).encode_to(&mut blob);
            for record in item.iter() {
                record.encode_to(&mut blob);
            }
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for ReadyQueue {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(ReadyQueue {
            queue: {
                let mut ready = ReadyQueue::default();
                for item in ready.queue.iter_mut() {
                    let len = decode_unsigned(blob)?;
                    for _ in 0..len {
                        item.push(ReadyRecord::decode(blob)?);
                    }
                }
                ready.queue
            }
        })
    }
}

impl Encode for AccumulatedHistory {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::with_capacity(EPOCH_LENGTH * std::mem::size_of::<WorkPackageHash>());

        for item in self.queue.iter() {
            encode_unsigned(item.len()).encode_to(&mut blob);
            for hash in item.iter() {
                hash.encode_to(&mut blob);
            }
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AccumulatedHistory {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(AccumulatedHistory {
            queue: {
                let mut history = AccumulatedHistory::default();
                for item in history.queue.iter_mut() {
                    let len = decode_unsigned(blob)?;
                    for _ in 0..len {
                        item.push(WorkPackageHash::decode(blob)?);
                    }
                }
                history.queue
            }
        })
    }
}

impl Encode for AlwaysAccumulateMapItem {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.id.encode_to(&mut blob);
        self.gas.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for AlwaysAccumulateMapItem {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(AlwaysAccumulateMapItem {
            id: ServiceId::decode(blob)?,
            gas: Gas::decode(blob)?,
        })
    }
}

impl Encode for Vec<AlwaysAccumulateMapItem> {
    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        encode_unsigned(self.len()).encode_to(&mut blob);
        for item in self.iter() {
            item.encode_to(&mut blob);
        }

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for Vec<AlwaysAccumulateMapItem> {
    
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        let len = decode_unsigned(blob)?;
        let mut items = Vec::with_capacity(len);

        for _ in 0..len {
            items.push(AlwaysAccumulateMapItem::decode(blob)?);
        }

        Ok(items)
    }
}

impl Encode for OutputAccumulation {
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        match self {
            OutputAccumulation::Ok(output) => {
                blob.push(0);
                output.encode_to(&mut blob);
            },
            OutputAccumulation::Err( ) => {
                blob.push(1);
            }
        }

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for OutputAccumulation {
    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        match blob.read_byte()? {
            0 => Ok(OutputAccumulation::Ok(AccumulateRoot::decode(blob)?)),
            1 => Ok(OutputAccumulation::Err( )),
            _ => Err(ReadError::InvalidData),
        }
    }
}