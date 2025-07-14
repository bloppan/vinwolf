use crate::jam_types::{ServiceId, OpaqueHash, Gas, WorkResult, WorkExecResult, WorkExecError, RefineLoad};
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::generic::{encode_unsigned, decode_unsigned};

impl Encode for WorkResult {

    fn encode(&self) -> Vec<u8> {

        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkResult>());
        self.service.encode_size(4).encode_to(&mut blob);
        self.code_hash.encode_to(&mut blob);
        self.payload_hash.encode_to(&mut blob);
        self.gas.encode_size(8).encode_to(&mut blob);

        self.result[0].encode_to(&mut blob);

        if self.result[0] == 0 {
            let result_len = encode_unsigned(self.result.len() - 1);
            result_len.encode_to(&mut blob);
            self.result[1..].encode_to(&mut blob);
        } 
        
        self.refine_load.encode_to(&mut blob);
        
        return blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkResult {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkResult {
            service: ServiceId::decode(blob)?,
            code_hash: OpaqueHash::decode(blob)?,
            payload_hash: OpaqueHash::decode(blob)?,
            gas: Gas::decode(blob)?,
            result: {
                let mut result: Vec<u8> = vec![];
                let exec_result = blob.read_byte()?;
                exec_result.encode_to(&mut result);
                
                match exec_result {
                    0 => {
                        let len = decode_unsigned(blob)?;
                        result.extend_from_slice(&blob.read_bytes(len)?);
                        WorkExecResult::Ok(result.clone())
                    },
                    1 => WorkExecResult::Error(WorkExecError::OutOfGas),
                    2 => WorkExecResult::Error(WorkExecError::Panic),
                    3 => WorkExecResult::Error(WorkExecError::BadNumberExports),
                    4 => WorkExecResult::Error(WorkExecError::ServiceCodeNotAvailableForLookup),
                    5 => WorkExecResult::Error(WorkExecError::BadCode),
                    6 => WorkExecResult::Error(WorkExecError::CodeOversize),
                    _ => { 
                        log::error!("Invalid value in WorkExecResult: {}", exec_result);
                        return Err(ReadError::InvalidData);
                    }
                };
                result
            },
            refine_load: RefineLoad::decode(blob)?,
        })
    }  
}

impl Encode for RefineLoad {

    fn encode(&self) -> Vec<u8> {
        let mut blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<Self>());
        
        encode_unsigned(self.gas_used as usize).encode_to(&mut blob);
        encode_unsigned(self.imports as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_count as usize).encode_to(&mut blob);
        encode_unsigned(self.extrinsic_size as usize).encode_to(&mut blob);
        encode_unsigned(self.exports as usize).encode_to(&mut blob);

        return blob;
    }
    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for RefineLoad {

    fn decode(blob: &mut BytesReader) -> Result<Self, ReadError> {
        Ok(RefineLoad {
            gas_used: decode_unsigned(blob)? as u64,
            imports: decode_unsigned(blob)? as u16,
            extrinsic_count: decode_unsigned(blob)? as u16,
            extrinsic_size: decode_unsigned(blob)? as u32,
            exports: decode_unsigned(blob)? as u16,
        })
    }
}
