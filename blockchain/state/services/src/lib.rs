use jam_types::{Preimage, ProcessError, OutputPreimages, ServiceAccounts, TimeSlot};
use block::extrinsic;
use utils::log;

pub fn process(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &[Preimage]
) -> Result<OutputPreimages, ProcessError> {

    log::debug!("Process the preimages extrinsic");

    if preimages_extrinsic.len() == 0 {
        log::debug!("No preimages to process");
        return Ok(OutputPreimages::Ok());
    }
    
    extrinsic::preimages::process(preimages_extrinsic, services, post_tau)?;

    log::debug!("Preimages extrinsic processed successfully");
    Ok(OutputPreimages::Ok())
}

