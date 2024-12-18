use crate::types::{ServiceId, OpaqueHash, Gas, WorkResult, WorkExecResult};
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};

impl Encode for WorkResult {

    fn encode(&self) -> Vec<u8> {

        let mut work_res_blob: Vec<u8> = Vec::with_capacity(std::mem::size_of::<WorkResult>());

        self.service.encode_size(4).encode_to(&mut work_res_blob);
        self.code_hash.encode_to(&mut work_res_blob);
        self.payload_hash.encode_to(&mut work_res_blob);
        self.gas.encode_size(8).encode_to(&mut work_res_blob);

        let mut result = BytesReader::new(&self.result);
        let exec_result = decode_unsigned(&mut result).expect("Error decoding exec_result in WorkResult");
        encode_unsigned(exec_result).encode_to(&mut work_res_blob);

        if exec_result == 0 {
            let len = decode_unsigned(&mut result).expect("Error decoding len in WorkResult");
            encode_unsigned(len).encode_to(&mut work_res_blob);
            let start_ok_data = result.get_position();
            self.result[start_ok_data..].encode_to(&mut work_res_blob);
        }
        
        return work_res_blob;
    }

    fn encode_to(&self, into: &mut Vec<u8>) {
        into.extend_from_slice(&self.encode());
    }
}

impl Decode for WorkResult {

    fn decode(work_result_blob: &mut BytesReader) -> Result<Self, ReadError> {

        Ok(WorkResult {
            service: ServiceId::decode(work_result_blob)?,
            code_hash: OpaqueHash::decode(work_result_blob)?,
            payload_hash: OpaqueHash::decode(work_result_blob)?,
            gas: Gas::decode(work_result_blob)?,
            result: {
                let mut result: Vec<u8> = vec![];
                let exec_result = work_result_blob.read_byte()?;
                exec_result.encode_to(&mut result);
                
                match exec_result {
                    0 => {
                        let len = decode_unsigned(work_result_blob)?;
                        encode_unsigned(len).encode_to(&mut result);
                        result.extend_from_slice(&work_result_blob.read_bytes(len)?);
                        WorkExecResult::Ok
                    },
                    1 => WorkExecResult::OutOfGas,
                    2 => WorkExecResult::Panic,
                    3 => WorkExecResult::BadCode,
                    4 => WorkExecResult::CodeOversize,
                    _ => { 
                        println!("Invalid value in WorkExecResult: {}", exec_result);
                        WorkExecResult::UnknownError
                    }
                };
                result
            },
        })
    }  
}

impl WorkResult {

    pub fn decode_len(work_result_blob: &mut BytesReader) -> Result<Vec<Self>, ReadError> {

        let num_results = decode_unsigned(work_result_blob)?;
        let mut results: Vec<WorkResult> = Vec::with_capacity(num_results);

        for _ in 0..num_results {
            let work_result = WorkResult::decode(work_result_blob)?;
            results.push(work_result);
        }

        Ok(results)
    }

    pub fn encode_len(results: &[WorkResult]) -> Vec<u8> {
        
        let mut encoded: Vec<u8> = Vec::new();
        encode_unsigned(results.len()).encode_to(&mut encoded);

        for result in results {
            result.encode_to(&mut encoded);
        }

        return encoded;
    }
}

