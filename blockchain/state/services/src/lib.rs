
use jam_types::{ProcessError, OutputPreimages, ServiceAccounts, TimeSlot};
use block::PreimagesExtrinsic;

pub fn process(
    services: &mut ServiceAccounts, 
    post_tau: &TimeSlot, 
    preimages_extrinsic: &PreimagesExtrinsic
) -> Result<OutputPreimages, ProcessError> {

    log::debug!("Process the preimages extrinsic");

    if preimages_extrinsic.preimages.len() == 0 {
        log::debug!("No preimages to process");
        return Ok(OutputPreimages::Ok());
    }
    
    preimages_extrinsic.process(services, post_tau)?;

    log::debug!("Preimages extrinsic processed successfully");
    Ok(OutputPreimages::Ok())
}

