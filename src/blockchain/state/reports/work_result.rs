use crate::constants::{WORK_REPORT_GAS_LIMIT, MAX_WORK_ITEMS};
use crate::types::{Gas, WorkResult, ReportErrorCode};
use crate::blockchain::state::ProcessError;
use crate::blockchain::state::get_service_accounts;

impl WorkResult {

    pub fn process(results: &[WorkResult]) -> Result<usize, ProcessError> {

        log::debug!("Processing work results");

        if results.len() < 1 {
            log::error!("No results");
            return Err(ProcessError::ReportError(ReportErrorCode::NoResults));
        }

        if results.len() > MAX_WORK_ITEMS {
            log::error!("Too many results: {:?}", results.len());
            return Err(ProcessError::ReportError(ReportErrorCode::TooManyResults));
        }

        let services = get_service_accounts();
        let mut total_accumulation_gas: Gas = 0;
        
        //let service_map: std::collections::HashMap<_, _> = services.0.iter().map(|s| (s.id, s)).collect();
        let mut results_size = 0;

        for result in results.iter() {
            if let Some(service) = services.get(&result.service) {
                // We require that all work results within the extrinsic predicted the correct code hash for their 
                // corresponding service
                if result.code_hash != service.code_hash {
                    log::error!("Bad code hash 0x{} != 0x{}", crate::print_hash!(result.code_hash), crate::print_hash!(service.code_hash));
                    return Err(ProcessError::ReportError(ReportErrorCode::BadCodeHash));
                }
                // We require that the gas allotted for accumulation of each work item in each work-report respects 
                // its service's minimum gas requirements
                // TODO revisar esto a ver si en realidad es este gas
                if result.gas < service.acc_min_gas {
                    log::error!("Service item gas too low: {:?}. The min gas required is {:?}", result.gas, service.acc_min_gas);
                    return Err(ProcessError::ReportError(ReportErrorCode::ServiceItemGasTooLow));
                }
                total_accumulation_gas += result.gas;

                if result.result[0] == 0 {
                    results_size += result.result.len() - 1;
                }
            } else {
                log::error!("Bad service id: {:?}", result.service);
                return Err(ProcessError::ReportError(ReportErrorCode::BadServiceId));
            }
        }

        // We also require that all work-reports total allotted accumulation gas is no greater than the WORK_REPORT_GAS_LIMIT
        if total_accumulation_gas > WORK_REPORT_GAS_LIMIT {
            log::error!("Work report gas too high: {:?}. The work report gas limit is {:?}", total_accumulation_gas, WORK_REPORT_GAS_LIMIT);
            return Err(ProcessError::ReportError(ReportErrorCode::WorkReportGasTooHigh));
        }

        log::debug!("Work results processed successfully");
        return Ok(results_size);
    }
}
