use crate::constants::WORK_REPORT_GAS_LIMIT;
use crate::types::{Gas, WorkResult, ReportErrorCode};
use crate::blockchain::state::ProcessError;
use crate::blockchain::state::services::get_services_state;

impl WorkResult {

    pub fn process(results: &[WorkResult]) -> Result<usize, ProcessError> {

        if results.len() < 1 {
            return Err(ProcessError::ReportError(ReportErrorCode::NoResults));
        }

        if results.len() > 4 {
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyResults));
        }

        let services = get_services_state();
        let mut total_accumulation_gas: Gas = 0;
        
        let service_map: std::collections::HashMap<_, _> = services.0.iter().map(|s| (s.id, s)).collect();
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

                if result.result[0] == 0 {
                    results_size += result.result.len() - 1;
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
