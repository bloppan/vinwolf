use crate::constants::WORK_REPORT_GAS_LIMIT;
use crate::types::{ServiceId, OpaqueHash, Gas, WorkResult, WorkExecResult};
use crate::blockchain::state::ProcessError;
use crate::blockchain::state::services::get_services_state;
use crate::utils::codec::{Encode, EncodeSize, Decode, BytesReader, ReadError};
use crate::utils::codec::{encode_unsigned, decode_unsigned};
use crate::utils::codec::work_report::ReportErrorCode;

impl WorkResult {

    pub fn process(results: &[WorkResult]) -> Result<usize, ProcessError> {

        if results.len() < 1 {
            return Err(ProcessError::ReportError(ReportErrorCode::NoResults));
        }

        if results.len() > 4 {
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyResults));
        }

        let list = get_services_state();
        let mut total_accumulation_gas: Gas = 0;
        
        let service_map: std::collections::HashMap<_, _> = list.services.iter().map(|s| (s.id, s)).collect();
        let mut results_size = 0;

        for result in results.iter() {
            if let Some(service) = service_map.get(&result.service) {
                // We require that all work results within the extrinsic predicted the correct code hash for their 
                // corresponding service
                if result.code_hash != service.info.code_hash {
                    return Err(ProcessError::ReportError(ReportErrorCode::BadCodeHash));
                }
                // We require that the gas allotted for accumulation of each work item in each work-report respects 
                // its service's minimum gas requirements
                if result.gas < service.info.min_item_gas {
                    return Err(ProcessError::ReportError(ReportErrorCode::ServiceItemGasTooLow));
                }
                total_accumulation_gas += result.gas;
               
                let mut result = BytesReader::new(&result.result);
                let exec_result = result.read_byte().map_err(ProcessError::ReadError)?;
                if exec_result == 0 {
                    results_size += decode_unsigned(&mut result).map_err(ProcessError::ReadError)?;
                }
            } else {
                return Err(ProcessError::ReportError(ReportErrorCode::BadServiceId));
            }
        }

        // We also require that all work-reports total allotted accumulation gas is no greater than the WORK_REPORT_GAS_LIMIT
        if total_accumulation_gas > WORK_REPORT_GAS_LIMIT {
            return Err(ProcessError::ReportError(ReportErrorCode::WorkReportGasTooHigh));
        }

        return Ok(results_size);
    }
}


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

