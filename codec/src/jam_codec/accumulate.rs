use constants::node::{EPOCH_LENGTH, TRANSFER_MEMO_SIZE};
use jam_types::{
    AccumulateRoot, AccumulatedHistory, OutputAccumulation, ReadyQueue, ReadyRecord, WorkPackageHash, WorkReport, AccumulationOperand, 
    DeferredTransfer, ServiceId, Balance, Gas
};
use crate::{BytesReader, Decode, DecodeLen, Encode, EncodeLen, ReadError};
use crate::generic_codec::{encode_unsigned, decode_unsigned};

impl Encode for DeferredTransfer {

    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();
        
        self.from.encode_to(&mut blob);
        self.to.encode_to(&mut blob);
        self.amount.encode_to(&mut blob);
        self.memo.encode_to(&mut blob);
        self.gas_limit.encode_to(&mut blob);

        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for DeferredTransfer {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        
        Ok(DeferredTransfer {
            from: ServiceId::decode(blob)?,
            to: ServiceId::decode(blob)?,
            amount: Balance::decode(blob)?,
            memo: blob.read_bytes(TRANSFER_MEMO_SIZE)?.to_vec(),
            gas_limit: Gas::decode(blob)?,
        })
    }
}

impl Encode for ReadyRecord {

    fn encode(&self) -> Vec<u8> {

        let mut blob = Vec::new();

        self.report.encode_to(&mut blob);
        self.dependencies.encode_len().encode_to(&mut blob);

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
            dependencies: Vec::<WorkPackageHash>::decode_len(blob)?,
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

impl Encode for AccumulationOperand {
    
    fn encode(&self) -> Vec<u8> {
        
        let mut blob = Vec::new();

        self.code_hash.encode_to(&mut blob);
        self.exports_root.encode_to(&mut blob);
        self.authorizer_hash.encode_to(&mut blob);
        self.payload_hash.encode_to(&mut blob);
        
        encode_unsigned(self.gas_limit as usize).encode_to(&mut blob);

        self.result[0].encode_to(&mut blob);
        if self.result[0] == 0 {
            let result_len = encode_unsigned(self.result.len() - 1);
            result_len.encode_to(&mut blob);
            self.result[1..].encode_to(&mut blob);
        } 

        self.auth_trace.encode_len().encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
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